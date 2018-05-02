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

use libc::c_uint;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub type sudo_printf_non_null_t = unsafe extern "C" fn(
    msg_type:   ::std::os::raw::c_int,
    fmt: *const ::std::os::raw::c_char,
    ...
) -> ::std::os::raw::c_int;

pub const SUDO_API_VERSION: c_uint =
    SUDO_API_VERSION_MAJOR << 16 | SUDO_API_VERSION_MINOR;
