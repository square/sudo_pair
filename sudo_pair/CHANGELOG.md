# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New `groups_exempted` and `groups_enforced` alternatives to `gids_exempted` and `gids_enforced`.

### Fixed
- The `-u` and `-g` flags can now both be used in exempt sessions.

### Changed
- Builds using Rust 2018
- Errors are handled through the `failure` crate rather than `error-chain`.

## [0.11.1] - 2018-06-08

### Fixed
- The approval command is once again implicitly whitelisted, this was unintentionally removed when adding support for obeying `log_output` hinting in `/etc/sudoers`.

## [0.11.0] - 2018-06-06

### Added
- Now obeys the `log_output` setting from `/etc/sudoers`. However, this renders this plugin completely nonfunctional unless this setting is enabled (or individual commands are opted in with the `LOG_OUTPUT:` tag).

### Removed
- Removed the short-lived `whitelist` setting in favor of simply honoring `log_output`.

## [0.10.0] - 2018-06-05

### Added
- New `whitelist` plugin option allows for naming binaries to be exempt from requiring a pair.

### Changed
- No longer fails to build on warnings, unless being run in CI

## [0.9.2] - 2018-05-18

### Added
- No longer forbids redirecting standard out and standard error

### Changed
- Prompt is rendered directly to the user's TTY when possible

### Fixed
- Output sent to the plugin printf function is sent with `write_all` for technical correctness (although AFAICT this is unnecessary in practice)

## [0.9.1] - 2018-05-08

### Security
- Ensure approval sockets aren't created with the primary group of the new user
- Print all the arguments passed to the command being `sudo`ed (thanks [`/u/__xor__`](https://www.reddit.com/r/rust/comments/8hppka/sudo_pair_090_released/dymsev8/))

### Fixed
- Rolled back the minimum plugin API version to 1.9; it was mistakenly bumped to 1.12 when support for 1.12 was added

## 0.9.0 - 2018-05-07

### Added
- First public release, stabilization pending feedback from the community

[Unreleased]: https://github.com/square/sudo_pair/compare/sudo_pair-v0.11.0...master
[0.11.1]:     https://github.com/square/sudo_pair/compare/sudo_pair-v0.11.0...sudo_pair-v0.11.1
[0.11.0]:     https://github.com/square/sudo_pair/compare/sudo_pair-v0.10.0...sudo_pair-v0.11.0
[0.10.0]:     https://github.com/square/sudo_pair/compare/sudo_pair-v0.9.2...sudo_pair-v0.10.0
[0.9.2]:      https://github.com/square/sudo_pair/compare/sudo_pair-v0.9.1...sudo_pair-v0.9.2
[0.9.1]:      https://github.com/square/sudo_pair/compare/sudo_pair-v0.9.0...sudo_pair-v0.9.1
