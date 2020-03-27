# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.1] - 2020-03-27

### Fixed

- [Fixed on 32-bit architectures][issue-59]; bindgen output cannot be
  committed directly, since it's architecture-dependent

[issue-59]: https://github.com/square/sudo_pair/issues/59

## [1.2.0] - 2020-03-26

### Changed
- Builds using Rust 2018
- No longer fails to build on warnings, unless being run in CI
- Bindgen-generated bindings are committed directly so we can remove
  bindgen from the list of build dependencies

## [1.1.0] - 2018-05-18

## Changed
- Updated to use bindgen 0.37, which changes the mutability of some pointer parameters

## [1.0.1] - 2018-05-08

### Fixed
- Preferentially use bundled sudo_plugin.h

## 1.0.0 - 2018-05-07

### Added
- Bindings automatically generated for [sudo_plugin(8)](https://www.sudo.ws/man/1.8.22/sudo_plugin.man.html)
- Provides default `sudo_plugin.h` which will be used if none is found on the system

[Unreleased]: https://github.com/square/sudo_pair/compare/sudo_plugin-sys-v1.2.1...master
[1.2.1]:      https://github.com/square/sudo_pair/compare/sudo_plugin-sys-v1.2.0...sudo_plugin-sys-v1.2.1
[1.2.0]:      https://github.com/square/sudo_pair/compare/sudo_plugin-sys-v1.1.0...sudo_plugin-sys-v1.2.0
[1.1.0]:      https://github.com/square/sudo_pair/compare/sudo_plugin-sys-v1.0.1...sudo_plugin-sys-v1.1.0
[1.0.1]:      https://github.com/square/sudo_pair/compare/sudo_plugin-sys-v1.0.0...sudo_plugin-sys-v1.0.1
