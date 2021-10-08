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
//! # Example:
//!
//! ```rust
//! use sudo_plugin_sys as sudo;
//! use std::os::raw;
//!
//! /// Plugins must be exposed as `pub static mut` and tagged with the
//! /// `no_mangle` attribute. The name of the variable it is assigned to will
//! /// be the `symbol_name` to be used in [`sudo.conf`][man-sudo-conf].
//! ///
//! /// The `no_mangle` attribute ensures that the exported symbol name is the
//! /// same as its variable name. It also ensures that the item will be
//! /// exported to the compiled library even though we're not using it directly
//! /// ourselves.
//! ///
//! /// The `static mut` declaration is required to ensure that the plugin is
//! /// placed in a single precise memory location in the compiled library (as
//! /// opposed to `const`, which simply inlines the value wherever it's used).
//! /// It must be declared `mut` so that the contents are not placed into
//! /// read-only memory, as the sudo plugin interface may write values into
//! /// the plugin (e.g., the `event_alloc` field).
//! ///
//! /// [man-sudo-conf]: https://www.sudo.ws/man/sudo.conf.man.html
//! #[no_mangle]
//! pub static mut deny_everything: sudo::approval_plugin = sudo::approval_plugin {
//!     open:         Some(open),
//!     show_version: Some(show_version),
//!
//!     .. sudo::approval_plugin::empty()
//! };
//!
//! static mut sudo_conv:  sudo::sudo_conv_t   = None;
//! static mut sudo_print: sudo::sudo_printf_t = None;
//!
//! const err : &'static [u8] = b"deny_everything denied this command\0";
//!
//! unsafe extern "C" fn open(
//!     version:                     raw::c_uint,
//!     conversation:                sudo::sudo_conv_t,
//!     sudo_printf:                 sudo::sudo_printf_t,
//!     _settings:       *const *mut raw::c_char,
//!     _user_info:      *const *mut raw::c_char,
//!     _submit_optind:              raw::c_int,
//!     _submit_argv:    *const *mut raw::c_char,
//!     _submit_envp:    *const *mut raw::c_char,
//!     _plugin_options: *const *mut raw::c_char,
//!     errstr:          *mut *const raw::c_char,
//! ) -> raw::c_int {
//!     unsafe {
//!         sudo_conv  = conversation;
//!         sudo_print = sudo_printf;
//!     }
//!
//!     if version >= sudo::sudo_api_mkversion(1, 17) {
//!         *errstr = err.as_ptr().cast();
//!     }
//!
//!     return 0;
//! }
//!
//! unsafe extern "C" fn show_version(_verbose: raw::c_int) -> raw::c_int {
//!     let print = match sudo_print {
//!         Some(f) => f,
//!         None    => { return 0 },
//!     };
//!
//!     unsafe {
//!         (print)(
//!             sudo::SUDO_CONV_INFO_MSG as _,
//!             b"Deny Everything version 1.0\n\0".as_ptr().cast(),
//!         )
//!     };
//!
//!     return 1;
//! }
//! ```
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

#![warn(rustdoc::all)]

#![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::correctness)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::style)]

mod sys {
    // this entire module is unsafe code
    #![allow(unsafe_code)]

    // this entire module is generated code
    #![allow(missing_docs)]
    #![allow(non_camel_case_types)]

    // this entire module is generated code
    #![allow(clippy::similar_names)]
    #![allow(clippy::type_complexity)]

    // FIXME: https://github.com/rust-lang/rust-bindgen/issues/1651
    #![allow(deref_nullptr)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use sys::*;

use std::os::raw::c_uint;

/// Constructs a sudo API verson from a major version and a minor version.
///
/// # Example:
///
/// ```rust
/// use sudo_plugin_sys as sudo;
///
/// let v1_9  = sudo::sudo_api_mkversion(1, 9);
/// let v1_0  = sudo::sudo_api_mkversion(1, 0);
/// let v1_17 = sudo::sudo_api_mkversion(1, 17);
///
/// assert!(v1_0 < v1_9);
/// assert!(v1_0 < v1_17);
/// assert!(v1_9 < v1_17);
/// ```
#[must_use]
pub const fn sudo_api_mkversion(major: c_uint, minor: c_uint) -> c_uint {
    major << 16 | minor
}

/// Gets the major version component from a sudo API version.
///
/// # Example:
///
/// ```rust
/// use sudo_plugin_sys as sudo;
///
/// let v1_0  = sudo::sudo_api_mkversion(1, 0);
/// let v1_17 = sudo::sudo_api_mkversion(1, 17);
/// let v99_1 = sudo::sudo_api_mkversion(99, 1);
///
/// assert_eq!(1,  sudo::sudo_api_version_get_major(v1_0));
/// assert_eq!(1,  sudo::sudo_api_version_get_major(v1_17));
/// assert_eq!(99, sudo::sudo_api_version_get_major(v99_1));
/// ```
#[must_use]
pub const fn sudo_api_version_get_major(version: c_uint) -> c_uint {
    version >> 16
}

/// Gets the minor version component from a sudo API version.
///
/// # Example:
///
/// ```rust
/// use sudo_plugin_sys as sudo;
///
/// let v1_0  = sudo::sudo_api_mkversion(1, 0);
/// let v1_17 = sudo::sudo_api_mkversion(1, 17);
/// let v99_1 = sudo::sudo_api_mkversion(99, 1);
///
/// assert_eq!(0,  sudo::sudo_api_version_get_minor(v1_0));
/// assert_eq!(17, sudo::sudo_api_version_get_minor(v1_17));
/// assert_eq!(1,  sudo::sudo_api_version_get_minor(v99_1));
/// ```
#[must_use]
pub const fn sudo_api_version_get_minor(version: c_uint) -> c_uint {
    version & 0xffff
}

/// The version of the sudo API this extension supports.
pub const SUDO_API_VERSION: c_uint = sudo_api_mkversion(
    SUDO_API_VERSION_MAJOR,
    SUDO_API_VERSION_MINOR,
);

impl policy_plugin {
    const EMPTY : Self = Self  {
        type_:   SUDO_POLICY_PLUGIN,
        version: SUDO_API_VERSION,

        open:             None,
        close:            None,
        show_version:     None,
        check_policy:     None,
        list:             None,
        validate:         None,
        invalidate:       None,
        init_session:     None,
        register_hooks:   None,
        deregister_hooks: None,
        event_alloc:      None,
    };

    /// Returns an empty instance of this plugin that provides no
    /// implementations for any callback.
    #[must_use]
    pub const fn empty() -> Self { Self::EMPTY }
}

impl io_plugin {
    const EMPTY : Self = Self {
        type_:   SUDO_IO_PLUGIN,
        version: SUDO_API_VERSION,

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
        change_winsize:   None,
        log_suspend:      None,
        event_alloc:      None,
    };

    /// Returns an empty instance of this plugin that provides no
    /// implementations for any callback.
    #[must_use]
    pub const fn empty() -> Self { Self::EMPTY }
}

impl audit_plugin {
    const EMPTY : Self = Self {
            type_:   SUDO_AUDIT_PLUGIN,
            version: SUDO_API_VERSION,

            open:             None,
            close:            None,
            accept:           None,
            reject:           None,
            error:            None,
            show_version:     None,
            register_hooks:   None,
            deregister_hooks: None,
            event_alloc:      None,
    };

    /// Returns an empty instance of this plugin that provides no
    /// implementations for any callback.
    #[must_use]
    pub const fn empty() -> Self { Self::EMPTY }
}

impl approval_plugin {
    const EMPTY: Self = Self {
        type_:   SUDO_APPROVAL_PLUGIN,
        version: SUDO_API_VERSION,

        open:         None,
        close:        None,
        check:        None,
        show_version: None,
    };

    /// Returns an empty instance of this plugin that provides no
    /// implementations for any callback.
    #[must_use]
    pub const fn empty() -> Self { Self::EMPTY }
}
