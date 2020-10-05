use std::ops::{Index, IndexMut};
use std::fmt;
use std::fmt::Debug;
use std::io::{Write, BufReader, BufRead};
use std::fs::File;
use std::convert::TryInto;
use derive_more::{Add, AddAssign, Sub, SubAssign};

/// An address inside the SSBC's memory space.
/// The program, stack, ports, and program status word are all mapped inside of this.
#[derive(Default, Copy, Clone, Add, AddAssign, Sub, SubAssign)]
pub struct Addr(std::num::Wrapping<u16>);
impl Addr {
    /// Cast a u16 into an address.
    pub const fn from_u16(a: u16) -> Self {
        Addr(std::num::Wrapping(a))
    }
}
impl From<Addr> for u16 {
    fn from(addr: Addr) -> u16 {
        addr.0.0
    }
}
impl From<Addr> for usize {
    fn from(addr: Addr) -> usize {
        addr.0.0 as usize
    }
}
impl From<u16> for Addr {
    fn from(a: u16) -> Self {
        Addr::from_u16(a)
    }
}

impl Debug for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:#04x}", self.0.0))
    }
}


/// The four ports. A and C are output, B and D are input.
#[derive(Clone,Copy,Debug)]
pub enum Port{
    A,
    B,
    C,
    D,
}
impl Port {
    pub fn to_addr(self) -> Addr {
        use Port::*;
        Addr::from(match self{
            A => 0xFFFC,
            B => 0xFFFD,
            C => 0xFFFE,
            D => 0xFFFF,
        })
    }
}

/// The status word's address, 0xFFFB. It is one of: 0x80 (Z), 0x40(N), or 0x00
pub const PSW: Addr = Addr::from_u16(0xFFFB);
/// The length of the SSBC's memory.
const MEMORY_LENGTH: usize = u16::MAX as usize + 1;

/// The 64KiB of memory that the SSBC accesses.
#[derive(Clone)]
pub struct Memory(pub Box<[u8; MEMORY_LENGTH]>);
impl Memory {
    pub fn new() -> Self {
        Memory(Box::new([0; MEMORY_LENGTH]))
    }
    pub fn get(&self, address: Addr) -> u8 {
        *self.0.index(usize::from(address))
    }
    pub fn get_mut(&mut self, address: Addr) -> &mut u8 {
        self.0.index_mut(usize::from(address))
    }
    pub fn set(&mut self, address: Addr, value: u8) {
        *self.get_mut(address) = value;
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct Ssbc {
    memory: Memory,
    /// Program Counter. Set to 0x0000 by .reset()
    pc: Addr,
    /// Stack Pointer. Set to 0xFFFA by .reset()
    sp: Addr,
    /// Fault flag is raised when an instruction is invalid.
    fault: bool,
    /// Halt flag is raised by halt instruction.
    halt: bool,

}

impl Ssbc {
    /// Retrieves the Program Status Word.
    pub fn get_psw(&self) -> u8 {
        self.memory.get(PSW)
    }
    /// Clears flags and program counter, sets stack pointer to 0xFFFA
    pub fn reset(&mut self) {
        self.pc = 0x0000.into();
        self.sp = 0xFFFA.into();
        self.fault = false;
        self.halt = false;
    }
    /// Read a single byte from the program counter, moving the pc 1 forward
    fn read_ir(&mut self) -> u8 {
        let ir = self.memory.get(self.pc);
        self.pc += 1.into();
        ir
    }
    /// Read two bytes at the program counter, moving the pc 2 forward
    fn read_ext(&mut self) -> u16 {
        let hi = self.memory.get(self.pc)   as u16;
        let lo = self.memory.get(self.pc+1.into()) as u16;
        self.pc += 2.into();
        hi*0x100+lo
    }
    fn update_psw(&mut self, val: u8) {
        self.memory.set(
            PSW,
            if val>128{ 0x40 }else if val==0 { 0x80 } else { 0x00 }
        );
    }
    /// Steps by a single instruction
    /// Referred to as "break" in the CLI
    pub fn step(&mut self) {
        if self.fault || self.halt {
            return;
        }
        match self.read_ir() {
            // nop
            0 => (),
            // halt
            1 => self.halt = true,
            // pushimm
            2 => {
                let ir = self.read_ir();
                self.memory.set(self.sp, ir);
                self.sp -= 1.into();
            },
            // pushext
            3 => {
                let ext = self.read_ext().into();
                self.memory.set(self.sp, self.memory.get(ext));
                self.sp -= 1.into();
            },
            // popinh
            4 => {
                self.sp += 1.into();
            },
            // popext
            5 => {
                let ext = self.read_ext();
                let pop = self.memory.get(self.sp+1.into());
                self.memory.set(ext.into(), pop);
                self.sp += 1.into();
            },
            // jnz
            6 => {
                let ext = self.read_ext();
                if self.memory.get(PSW) != 0x80 {
                    self.pc = ext.into();
                }
            },
            // jnn
            7 => {
                let ext = self.read_ext();
                if self.memory.get(PSW) != 0x40 {
                    self.pc = ext.into();
                }
            },
            // add
            8 => {
                let result = self.memory.get(self.sp+2.into()).wrapping_add(self.memory.get(self.sp+1.into()));
                self.memory.set(self.sp+2.into(), result);
                self.update_psw(result);
                self.sp += 1.into();
            },
            // sub
            9 => {
                let result = self.memory.get(self.sp+1.into()).wrapping_sub(self.memory.get(self.sp+2.into()));
                self.memory.set(self.sp+2.into(), result);
                self.update_psw(result);
                self.sp += 1.into();
            },
            // nor
            10 => {
                let bw_or = self.memory.get(self.sp+2.into()) | self.memory.get(self.sp+1.into());
                self.memory.set(self.sp+2.into(), !bw_or);
                self.sp+=1.into();
            },
            // fault
            _ => self.fault = true,
        }
    }
    /// Runs instructions, until halt instr or fault (invalid instr).
    pub fn run(&mut self) {
        while !self.fault && !self.halt {
            self.step();
        }
    }
}

#[derive(Default)]
pub struct SsbcCli {
    ssbc: Ssbc,
}

impl SsbcCli {
    pub fn new() -> Self {
        Self::default()
    }
    /// Repeatedly asks the operator for commands, until quit or EOF on stdin.
    pub fn repl(&mut self) {
        loop {
            Self::prompt();
            let mut command = String::new();
            std::io::stdin().read_line(&mut command)
             .expect("Couldn't read operator's command");
            match command.chars().next().unwrap_or(' ') {
                'R' => self.reset(),
                'b' => self.ssbc.step(),
                'r' => self.ssbc.run(),
                'A' => self.read_port(Port::A),
                'B' => self.write_port(Port::B),
                'C' => self.read_port(Port::C),
                'D' => self.write_port(Port::D),
                's' => self.status(),
                't' => self.top(),
                'p' => self.psw(),
                'q' => return,
                _ => println!("WARNING: Unknown command")
            }
        }
    }
    fn prompt() {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();

        writeln!(out, "+------------------------+ ").ok();
        writeln!(out, "|  R: RESET              | ").ok();
        writeln!(out, "|  b: BREAK              | ").ok();
        writeln!(out, "|  r: RUN                | ").ok();
        writeln!(out, "|  A: READ PORT A        | ").ok();
        writeln!(out, "|  B: WRITE PORT B       | ").ok();
        writeln!(out, "|  C: READ PORT C        | ").ok();
        writeln!(out, "|  D: WRITE PORT D       | ").ok();
        writeln!(out, "|  s: STATUS             | ").ok();
        writeln!(out, "|  t: TOP                | ").ok();
        writeln!(out, "|  p: PSW                | ").ok();
        writeln!(out, "|  q: QUIT               | ").ok();
        writeln!(out, "|                        | ").ok();
        writeln!(out, "|  Enter menu selection: | ").ok();
        writeln!(out, "+------------------------+ ").ok();
    }
    fn reset(&mut self) {
        self.ssbc.reset();
        // Load machine code from `mac`
        let mac = BufReader::new(File::open("mac").expect("Couldn't open `mac` machine code file!"));
        for (x, line) in mac.lines().filter_map(Result::ok).enumerate() {
            let x: u16 = match x.try_into() { Ok(x) => x, Err(_) => {println!("WARNING: Machine code exceeds memory size!"); return} };
            if line.len() >= 8 {
                let value = u8::from_str_radix(&line[0..8], 2).expect("Couldn't parse user input");
                self.ssbc.memory.set(x.into(), value);
            }
        }
    }
    fn read_port(&self, port: Port) {
        let value = self.ssbc.memory.get(port.to_addr());
        if value==0 {
            // weird perl interpreter thing where it
            //     prints blank instead of zero.
            println!("Port {:?} value:  ", port);
        } else {
            println!("Port {:?} value: {:08b} ", port, value);
        }
    }
    fn write_port(&mut self, port: Port) {
        print!("Enter Port D value in binary (8 bits) ");
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).expect("Couldn't read value for port");
        buffer.pop();
        let value = u8::from_str_radix(&buffer, 2).expect("Couldn't parse user input");
        self.ssbc.memory.set(port.to_addr(), value);
    }
    fn status(&self) {
        println!("Fault: {} ", if self.ssbc.fault {1}else{0} );
        println!(" Halt: {} ", if self.ssbc.halt {1}else{0} );
    }
    fn top(&self) {
        println!("Top of stack: {:08b}", self.ssbc.memory.get(self.ssbc.sp+1.into()));
    }
    fn psw(&self) {
        println!("PSW: {:08b}", self.ssbc.get_psw());
    }
}

fn main() {
    let mut ssbc_cli = SsbcCli::default();
    ssbc_cli.repl();
}
