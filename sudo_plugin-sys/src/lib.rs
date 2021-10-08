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

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

const SUDO_API_VERSION: c_uint =
    SUDO_API_VERSION_MAJOR << 16 | SUDO_API_VERSION_MINOR;

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
