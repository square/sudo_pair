//! description = "FFI wrapper around authoring a sudo_plugin"
//!
//! TODO: explain

#![deny(warnings)]

#![warn(anonymous_parameters)]
#![warn(box_pointers)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unstable_features)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]

// this entire crate is practically unsafe code
#![allow(unsafe_code)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(clippy))]
#![cfg_attr(feature="clippy", warn(clippy_pedantic))]

// TODO: disable
#![cfg_attr(feature="clippy", allow(missing_docs_in_private_items))]

pub extern crate sudo_plugin_sys;

extern crate libc;

#[macro_use]
extern crate error_chain;

#[macro_use]
pub mod macros;

pub mod plugin;
pub mod errors;

mod version;

pub use sudo_plugin_sys as sys;

pub use self::plugin::*;
pub use self::errors::*;
