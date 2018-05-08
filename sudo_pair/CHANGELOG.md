# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [0.9.1] - 2018-05-08

### Security
- Ensure approval sockets aren't created with the primary group of the new user
- Print all the arguments passed to the command being `sudo`ed (thanks [`/u/__xor__`](https://www.reddit.com/r/rust/comments/8hppka/sudo_pair_090_released/dymsev8/))

### Fixed
- Rolled back the minimum plugin API version to 1.9; it was mistakenly bumped to 1.12 when support for 1.12 was added

## 0.9.0 - 2018-05-07

### Added
- First public release, stabilization pending feedback from the community

[0.9.1]: https://github.com/square/sudo_pair/compare/sudo_pair-v0.9.0...sudo_pair-v0.9.1
