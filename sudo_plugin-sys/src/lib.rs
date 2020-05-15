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

#![warn(bad_style)]
#![warn(future_incompatible)]
#![warn(nonstandard_style)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(rustdoc)]
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
#![allow(trivial_casts)]
#![allow(trivial_numeric_casts)]

#![cfg_attr(feature="cargo-clippy", warn(clippy::all))]

// this entire crate is generated code
#![cfg_attr(feature="cargo-clippy", allow(clippy::similar_names))]
#![cfg_attr(feature="cargo-clippy", allow(clippy::type_complexity))]

use std::os::raw::{c_int, c_uint};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const SUDO_API_VERSION: c_uint =
    SUDO_API_VERSION_MAJOR << 16 | SUDO_API_VERSION_MINOR;

pub const SUDO_PLUGIN_OPEN_SUCCESS       : c_int =  1;
pub const SUDO_PLUGIN_OPEN_FAILURE       : c_int =  0;
pub const SUDO_PLUGIN_OPEN_GENERAL_ERROR : c_int = -1;
pub const SUDO_PLUGIN_OPEN_USAGE_ERROR   : c_int = -2;

pub const SUDO_PLUGIN_LOG_OK     : c_int = 1;
pub const SUDO_PLUGIN_LOG_REJECT : c_int = 0;
pub const SUDO_PLUGIN_LOG_ERROR  : c_int = -1;

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

    #[cfg(feature = "min_sudo_plugin_1_12")]
    change_winsize: None,
};
