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
* `defmt`: you can enable the [`defmt`](https://defmt.ferrous-systems.com/) feature to get a `defmt::debug!` call for every speed change.

## Examples
A simple example for the STM32F4 microcontrollers is [available](examples/stm32f4-single-motor-example/README.md).

## Changelog
For the changelog please see the dedicated [CHANGELOG.md](CHANGELOG.md).

## Roadmap to v1.0.0
This crate is already stable, however it's based on a release candidate version of [`embedded-hal`](https://github.com/rust-embedded/embedded-hal/),
making the API unstable (the change from 1.0.0-rc.1 to 1.0.0 of e-h will be breaking from a dependency management point of view).

See [the tracking issue](https://github.com/rust-embedded-community/tb6612fng-rs/issues/4) for the roadmap to v1.0.0.

## Minimum Supported Rust Version (MSRV)
This crate is guaranteed to compile on stable Rust 1.62 and up. It *might*
compile with older versions but that may change in any new patch release.
