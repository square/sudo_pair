// Copyright 2018 Square Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

//! This crate is a (lighly enhanced) set of bindgen-generated Rust FFI
//! bindings for the [`sudo_plugin(8)`][sudo_plugin] facility. In
//! general, it is expected that end-users will prefer to use the
//! handcrafted Rust wrappers from the `sudo_plugin` crate which
//! accompanies this project.
//!
//! [sudo_plugin]: https://www.sudo.ws/man/1.8.22/sudo_plugin.man.html

#![warn(future_incompatible)]
#![warn(nonstandard_style)]
#![warn(rust_2021_compatibility)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(unused)]

#![warn(bare_trait_objects)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(single_use_lifetimes)]
#![warn(unreachable_pub)]
#![warn(unstable_features)]
#![warn(unused_import_braces)]
#![warn(unused_lifetimes)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]

// this entire crate is unsafe code
#![allow(unsafe_code)]

// this entire crate is generated code
#![allow(missing_docs)]
#![allow(non_camel_case_types)]

#![warn(rustdoc::all)]

#![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::correctness)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::style)]

// this entire crate is generated code
#![allow(clippy::similar_names)]
#![allow(clippy::type_complexity)]

// FIXME: https://github.com/rust-lang/rust-bindgen/issues/1651
#![allow(deref_nullptr)]

use std::os::raw::c_uint;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const SUDO_API_VERSION: c_uint =
    SUDO_API_VERSION_MAJOR << 16 | SUDO_API_VERSION_MINOR;

pub const IO_PLUGIN_EMPTY : io_plugin = io_plugin {
    type_:            SUDO_IO_PLUGIN,
    version:          SUDO_API_VERSION,
    open:             None,
    close:            None,
    show_version:     None,
    log_ttyin:        None,
    log_ttyout:       None,
    log_stdin:        None,
    log_stdout:       None,
    log_stderr:       None,
    register_hooks:   None,
    deregister_hooks: None,

    #[cfg(feature = "change_winsize")]
    change_winsize: None,
};
