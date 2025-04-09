# Rust Driver for TB6612FNG Motor Driver
[![CI](https://github.com/rust-embedded-community/tb6612fng-rs/actions/workflows/CI.yml/badge.svg)](https://github.com/rust-embedded-community/tb6612fng-rs/actions/workflows/CI.yml)
[![Crates.io](https://img.shields.io/crates/v/tb6612fng)](https://crates.io/crates/tb6612fng)
![Licenses](https://img.shields.io/crates/l/tb6612fng)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

This is a `no_std` driver for the [TB6612FNG motor driver](https://www.sparkfun.com/datasheets/Robotics/TB6612FNG.pdf) as can e.g. be found on the corresponding [SparkFun module](https://www.sparkfun.com/products/14450).

Note that this work is not affiliated with any of the vendors of the controller or controller boards.

The motor driver itself supports two motors and has a standby pin which controls both at the same time.
The crate can be either used to control a single motor (using the `Motor` struct directly) or
to control both motors (using the `Tb6612fng` struct) - the latter also supports using the standby functionality.

See the documentation for usage examples.

## When to use what
* You plan on using both motors and the standby feature: use `Tb6612fng`
* You plan on using both motors without the standby feature: use two separate `Motor`s
* You plan on using a single motor with the standby feature: use `Motor` and control the standby pin manually
* You plan on using a single motor without the standby feature: use `Motor`

## Optional features
* `defmt`: you can enable this feature to get a `defmt::Format` implementation for all structs & enums in this crate and a `defmt::trace` call for every speed change.

## Examples
A simple example for the STM32F4 microcontrollers is [available](examples/stm32f4-single-motor-example/README.md).

## Changelog
For the changelog please see the dedicated [CHANGELOG.md](CHANGELOG.md).

## Minimum Supported Rust Version (MSRV)
This crate is guaranteed to compile on stable Rust 1.81 and up. It *might*
compile with older versions but that may change in any new patch release.

## License
Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
