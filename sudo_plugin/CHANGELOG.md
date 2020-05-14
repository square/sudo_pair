# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Incorporated `plugin_name` and `plugin_version` into the `Plugin` struct
- `Drop::drop` is called on plugins when sudo exits

### Changed
- Wrapped `printf_facility` plugin argument into a dedicated `PrintFacility`
  struct to wrap all user communication.
- Moved much of the work done in the `sudo_io_plugin!` macro to non-macro code.
- Almost complete rewrite of the internals to be more Rust-like, and use traits
  to reduce the amount of macro magic necessary.

## [1.2.0] - 2020-03-26

### Added
- Automatic support for `-V` flag, which prints the version of `sudo` and any
  active plugins

### Changed
- Builds using Rust 2018
- No longer fails to build on warnings, unless being run in CI
- Allows plugins to use any error library they wish, as long as the error
  types returned in `Result`s implement `Into<sudo_plugin::errors::Error>`.

## [1.1.0] - 2018-05-18

### Added
- Support writing directly to the user's TTY

### Changed
- `UserInfo::tty` is now a `PathBuf` instead of a `String`.
- Depends on `sudo_plugin-sys` ~1.1, which changed mutability of pointer arguments due to bindgen 0.37

## 1.0.0 - 2018-05-07

### Added
- Macros to simplify writing sudo plugins
- Full compatibility with plugin API versions up to 1.12

[Unreleased]: https://github.com/square/sudo_pair/compare/sudo_pair-v1.2.0...master
[1.2.0]:      https://github.com/square/sudo_pair/compare/sudo_pair-v1.1.0...sudo_pair-v1.2.0
[1.1.0]:      https://github.com/square/sudo_pair/compare/sudo_pair-v1.0.0...sudo_pair-v1.1.0
