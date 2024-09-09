# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate
### Added

* Add a `current_standby()` method to check if the driver is currently in standby mode.

### Changed

* `Motor::new()` and `Driver::new()` methods now set the outputs upon their
  initialisation to the documented defaults.
* `Motor::new()` and `Driver::new()` methods now also return errors if they fail
  to set their outputs upon initialisation.
* **Breaking**: update to `embedded-hal` 1.0
* **Breaking**: Renamed error types to their struct names
* **Breaking**: Renamed `DriveCommand::Backwards` to `DriveCommand::Backward` to match
  `DriveCommand::Forward`

### Removed

* Removed the `drive_forward`, `drive_backward`, `stop` and `brake`
  functions as they are duplicates to the `drive` function with the
  different enum variants and make the API surface larger
* Removed the `defmt` feature: it was only used for debugging and since the `enum`s & `struct`s implement `Debug`
  consuming code can use `defmt::Debug2Format` when needed. The single `defmt::debug!` statement in `Motor::drive` was
  not very helpful anyway if two motors were connected

## [0.2.0] - 2023-11-28

### Changed

* Due to dependency updates the MSRV has been updated from 1.60 to 1.63. This
  should only be relevant if you use the `defmt` feature, but we now only test
  with 1.63 and not older releases, so it's not guaranteed to work otherwise.
* Breaking: the API was migrated from `embedded-hal:0.2` to `embedded-hal:1.0.0-rc1`.
  If your HAL does not yet implement this, then please use the previous release of the library.

<!-- next-url -->
[Unreleased]: https://github.com/rust-embedded-community/tb6612fng-rs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/rust-embedded-community/tb6612fng-rs/compare/v0.1.1...v0.2.0
