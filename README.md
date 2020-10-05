# SSBC-rs
An interpreter for the SSBC.

For more information on the SSBC, see [Peter Walsh's Computer Architecture course](http://csci.viu.ca/~pwalsh/teaching/261/261/261.html),
or [a past student's online interpreter written in JavaScript](https://babakanoosh.github.io/Visual-SSBC/index.html).

## Differences
The SSBC has a different subtract instruction, depending on whether you're in CSCI 261 or CSCI 355,
See the Signed Mode section of this document for instructions on how to set the mode.

Telling ssbc.pl to run without first resetting enters an infinite loop, with no side affects.
Telling ssbc-rs to run without first resetting begins interpreting, which will likely NOP, unless there's been something written to ports B or D

Situations where multiple reads and/or writes are happening to the same location within an instruction may behave differently than ssbc.pl (untested).

Register overflow (stack pointer, program counter) wrap, whereas ssbc.pl likely crashes with an array index out of bounds error (untested).

## Compiling
On otter, run `./build.sh`
Elsewhere, I recommend using cargo: `cargo build`

### Signed Mode
By default, ssbc-rs uses two's complement subtraction, as the CSCI 355 SSBC does.

If you'd like to match the 261 SSBC's behaviour,
pass `--cfg 'feature="signedmagnitude_sub"'` to build.sh when building on otter
or pass `--features=signedmagnitude_sub` when you're elsewhere.


## Usage
Use the ssbc-rs binary as you would `ssbc.pl`.
If you'd like to run the SSBC's tests on ssbc-rs, you can modify `.batch_test` to run the ssbc-rs binary instead of `ssbc.pl`.
