# SSBC-rs
An interpreter for the SSBC.

For more information on the SSBC, see [Peter Walsh's Computer Architecture course](http://csci.viu.ca/~pwalsh/teaching/261/261/261.html),
or [a past student's online interpreter written in JavaScript](https://babakanoosh.github.io/Visual-SSBC/index.html).

## Differences
the `sub` instruction uses two's complement arithmetic rather than signed magnitude. I'm attempting to upstream this.

Telling ssbc.pl to run without first resetting enters an infinite loop, with no side affects.
Telling ssbc-rs to run without first resetting begins interpreting, which will likely NOP, unless there's been something written to ports B or D

Situations where multiple reads and/or writes are happening to the same location within an instruction may behave differently than ssbc.pl (untested).

Register overflow (stack pointer, program counter) wrap, whereas ssbc.pl likely crashes with an array index out of bounds error (untested).

## Compiling
On otter, run `rustc --edition 2018 ssbc.rs`.
Elsewhere, I recommend using cargo: `cargo build`

Note that the ssbc.rs file in the root may be slightly out of date with the authoritive src/main.rs file.

## Usage
Use the ssbc-rs binary as you would `ssbc.pl`.
If you'd like to run the SSBC's tests on ssbc-rs, you can modify `.batch_test` to run the ssbc-rs binary instead of `ssbc.pl`.
