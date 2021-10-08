# sudo_plugin-sys

[![Build Status](https://github.com/square/sudo_pair/actions/workflows/rust.yml/badge.svg?branch=master)][badge-build]
[![Docs](https://img.shields.io/docsrs/sudo_plugin-sys)][badge-docs]
[![Latest Version](https://img.shields.io/crates/v/sudo_plugin-sys.svg)][badge-crate]
[![License](https://img.shields.io/github/license/square/sudo_pair.svg)][license]

FFI definitions for the [sudo_plugin(8)][sudo_plugin_man] facility.

## Sudo Plugin API Version

This crate matches its version to the upstream [sudo_plugin(8)][sudo_plugin_man]
API version it implements and will use patch versions to indicate internal fixes
to the crate.

Historically, the `sudo_plugin(8)` binary interface has been
backwards-compatible with previous versions and respected semantic versioning
practices. Since we track the upstream version numbering scheme, we believe this
project can also respect semantic versioning but cannot make this a guarantee.

Later versions of this plugin *should* be backward-compatible with older
versions of the `sudo_plugin(8)` interface. However, the user of this API is
responsible for checking the sudo front-end API version before using certain
features. Please consult the [manpage][sudo_plugin_man] to identify which
features require version probing.

## Building

This crate includes pregenerated bindings for `x86`, `x86-64`, and `aarch64`
architectures and will use them by default when building with Cargo.

```sh
cargo build
```

For other architectures, you'll need to build with the `bindgen` feature
enabled. This will generate bindings from the copy of `sudo_plugin.h` bundled
with this library.

```sh
cargo build --features bindgen
```

As releases of this library are built with a specific version of the plugin API
in mind, we do not currently support building against external versions of this
header. Since newer versions of the sudo plugin interface are binary-compatible
with older versions (and vice versa), doing so should not be necessary. If you
find a use-case that requires this, please [let us know][new-issue].

## Contributions

Contributions are welcome!

As this project is operated under [Square's open source program][square-open-source],
new contributors will be asked to sign a [contributor license agreement][square-cla]
that ensures we can continue to develop, maintain, and release this project
openly.

## License

`sudo_plugin-sys` is distributed under the terms of the [Apache License, Version
2.0][license].

[badge-build]:        https://github.com/square/sudo_pair/actions/workflows/rust.yml
[badge-docs]:         https://docs.rs/sudo_plugin-sys
[badge-crate]:        https://crates.io/crates/sudo_plugin-sys
[license]:            https://github.com/square/sudo_pair/blob/master/LICENSE-APACHE
[new-issue]:          https://github.com/square/sudo_pair/issues/new
[square-cla]:         https://cla-assistant.io/square/sudo_pair
[square-open-source]: https://square.github.io/
[sudo_plugin_man]:    https://www.sudo.ws/man/sudo_plugin.man.html
