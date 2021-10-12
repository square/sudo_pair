//! An example minimal sudo approval plugin that denies all actions.
//!
//! # Usage
//!
//! ```
//! Plugin deny_everything deny_everything.so
//! ```

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
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
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

use sudo_plugin_sys as sudo;
use std::os::raw;

/// Plugins must be exposed as `pub static mut` and tagged with the
/// `no_mangle` attribute. The name of the variable it is assigned to will
/// be the `symbol_name` to be used in [`sudo.conf`][man-sudo-conf].
///
/// The `no_mangle` attribute ensures that the exported symbol name is the
/// same as its variable name. It also ensures that the item will be
/// exported to the compiled library even though we're not using it directly
/// ourselves.
///
/// The `static mut` declaration is required to ensure that the plugin is
/// placed in a single precise memory location in the compiled library (as
/// opposed to `const`, which simply inlines the value wherever it's used).
/// It must be declared `mut` so that the contents are not placed into
/// read-only memory, as the sudo plugin interface may write values into
/// the plugin (e.g., the `event_alloc` field).
///
/// [man-sudo-conf]: https://www.sudo.ws/man/sudo.conf.man.html
#[no_mangle]
pub static mut deny_everything: sudo::approval_plugin = sudo::approval_plugin {
    open:         Some(open),
    check:        Some(check),
    show_version: Some(show_version),

    .. sudo::approval_plugin::empty()
};

static mut SUDO_CONV:        sudo::sudo_conv_t   = None;
static mut SUDO_PRINT:       sudo::sudo_printf_t = None;
static mut SUDO_API_VERSION: Option<raw::c_uint> = None;

const VERSION_MSG: *const raw::c_char = b"Deny Everything version 1.0\n\0".as_ptr().cast();
const ERRSTR:      *const raw::c_char = b"deny_everything: denied\n\0"    .as_ptr().cast();

unsafe extern "C" fn open(
    version:                     raw::c_uint,
    conversation:                sudo::sudo_conv_t,
    sudo_printf:                 sudo::sudo_printf_t,
    _settings:       *const *mut raw::c_char,
    _user_info:      *const *mut raw::c_char,
    _submit_optind:              raw::c_int,
    _submit_argv:    *const *mut raw::c_char,
    _submit_envp:    *const *mut raw::c_char,
    _plugin_options: *const *mut raw::c_char,
    _errstr:         *mut *const raw::c_char,
) -> raw::c_int {
    SUDO_CONV        = conversation;
    SUDO_PRINT       = sudo_printf;
    SUDO_API_VERSION = Some(version);

    1
}

unsafe extern "C" fn check(
    _command_info: *const *mut raw::c_char,
    _run_argv:     *const *mut raw::c_char,
    _run_envp:     *const *mut raw::c_char,
    errstr:        *mut *const raw::c_char,
) -> raw::c_int {
    let print = match SUDO_PRINT {
        Some(f) => f,
        None    => { return 0 },
    };

    #[allow(clippy::cast_possible_wrap)]
    let _ = (print)(sudo::SUDO_CONV_INFO_MSG as _, ERRSTR);

    if SUDO_API_VERSION.map(|v| v >= sudo::sudo_api_mkversion(1, 17)) == Some(true) {
        *errstr = ERRSTR;
    }

    0
}

unsafe extern "C" fn show_version(_verbose: raw::c_int) -> raw::c_int {
    let print = match SUDO_PRINT {
        Some(f) => f,
        None    => { return 0 },
    };

    #[allow(clippy::cast_possible_wrap)]
    let _ = (print)(sudo::SUDO_CONV_INFO_MSG as _, VERSION_MSG);

    1
}
