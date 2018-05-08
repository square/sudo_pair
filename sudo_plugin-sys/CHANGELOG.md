# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2018-05-08

### Fixed
- Preferentially use bundled sudo_plugin.h

## [1.0.0] - 2018-05-07

### Added
- Bindings automatically generated for [sudo_plugin(8)](https://www.sudo.ws/man/1.8.22/sudo_plugin.man.html)
- Provides default `sudo_plugin.h` which will be used if none is found on the system
