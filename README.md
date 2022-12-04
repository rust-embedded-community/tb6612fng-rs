# Rust Driver for TB6612FNG Motor Driver
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
