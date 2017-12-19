//! description = "FFI wrapper around authoring a sudo_plugin"
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

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", warn(clippy))]
#![cfg_attr(feature = "clippy", warn(clippy_pedantic))]

extern crate libc;

use libc::c_uint;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const SUDO_API_VERSION: c_uint
    = SUDO_API_VERSION_MAJOR << 16
    | SUDO_API_VERSION_MINOR;
