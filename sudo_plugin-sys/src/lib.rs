//! description = "FFI wrapper around authoring sudo plugins"
//!
//! TODO: explain

#![deny(warnings)]

#![warn(anonymous_parameters)]
#![warn(box_pointers)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unstable_features)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]

#![allow(missing_docs)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]

#![cfg_attr(feature="cargo-clippy", warn(clippy))]
#![cfg_attr(feature="cargo-clippy", warn(clippy_pedantic))]
#![cfg_attr(feature="cargo-clippy", allow(similar_names))]
#![cfg_attr(feature="cargo-clippy", allow(type_complexity))]
#![cfg_attr(feature="cargo-clippy", allow(unseparated_literal_suffix))]

extern crate libc;

use libc::{c_int, c_uint};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub type sudo_printf_non_null_t = unsafe extern "C" fn(
    msg_type:   ::std::os::raw::c_int,
    fmt: *const ::std::os::raw::c_char,
    ...
) -> ::std::os::raw::c_int;

pub const SUDO_API_VERSION: c_uint =
    SUDO_API_VERSION_MAJOR << 16 | SUDO_API_VERSION_MINOR;

pub const SUDO_PLUGIN_OPEN_SUCCESS       : c_int =  1;
pub const SUDO_PLUGIN_OPEN_FAILURE       : c_int =  0;
pub const SUDO_PLUGIN_OPEN_GENERAL_ERROR : c_int = -1;
pub const SUDO_PLUGIN_OPEN_USAGE_ERROR   : c_int = -2;
