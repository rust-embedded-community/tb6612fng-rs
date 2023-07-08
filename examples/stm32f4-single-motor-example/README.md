# TB6612FNG Motor Driver Using Rust on NUCLEO-F401RE ARM32 Board
This example showcases how the [tb6612fng](https://crates.io/crates/tb6612fng) crate for the [SparkFun Motor Driver - Dual TB6612FNG (1A)](https://www.sparkfun.com/products/14451) can be used on an STM32F4 chip.
It uses [RTIC](https://rtic.rs/) (Real-Time Interrupt-driven Concurrency) underneath.

The example logs messages using [`defmt`](https://defmt.ferrous-systems.com/).

The example has been tested on a [ST Nucleo-F401RE](https://www.st.com/en/evaluation-tools/nucleo-f401re.html) development
board but should work on any STM32F4xx family microcontroller as long as the controller is connected on the following pins (or the code adapted accordingly):
* `AI1` on `PB5`
* `AI2` on `PB4`
* `PWMA` on `PB10`

Don't forget to pull the `STBY` pin high, otherwise nothing will happen!

Furthermore, the example uses a button connected on `PC13` which is e.g. present on the mentioned board.

The example continuously cycles through all speeds from full backwards to full forward (in 1% steps) and the button can be used
to stop (coast, first press), actively brake (second press) and drive again (third press).

## Prerequisites
1. Optional: ensure that the rust toolchain is up-to-date: `rustup update`
1. Install `probe-run`: `cargo install probe-run`
1. Install `flip-link`: `cargo install flip-link`
    * Note: `flip-link` is not strictly necessary for this example (it doesn't need
      stack protection), however it can be considered best practices to include it.
1. Install the cross-compile target: `rustup target add thumbv7em-none-eabihf`
1. Optional: install the LLVM tools: `rustup component add llvm-tools-preview`
1. Install the STLink drivers

## Build & Download to Board
1. Connect the board via USB
2. Run `cargo run` (the correct chip & target is already defined in `Cargo.toml` and `.cargo/config`)
3. Enjoy your running program :)
