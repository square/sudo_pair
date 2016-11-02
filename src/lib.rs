//! sudo IO-plugin to require a live human pair.
//!
//! TODO: explain

#![deny(warnings)]

#![warn(fat_ptr_transmutes)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unstable_features)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]

#![cfg_attr(feature="clippy", allow(unstable_features))]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(clippy))]
#![cfg_attr(feature="clippy", warn(clippy_pedantic))]

extern crate libc;

mod sudo;

use libc::{c_char, c_int, c_uint};

/// The exported plugin function that hooks into sudo.
#[cfg_attr(rustfmt, rustfmt_skip)]
pub static SUDO_PAIR_PLUGIN: sudo::io_plugin = sudo::io_plugin {
    type_:            sudo::SUDO_IO_PLUGIN,
    version:          sudo::SUDO_API_VERSION,
    open:             Some(open),
    close:            None,
    show_version:     None,
    log_ttyin:        None,
    log_ttyout:       None,
    log_stdin:        None,
    log_stdout:       None,
    log_stderr:       None,
    register_hooks:   None,
    deregister_hooks: None,
};

extern "C" fn open(
    _version: c_uint,
    _conversation: sudo::sudo_conv_t,
    _sudo_printf: sudo::sudo_printf_t,
    _settings: *const *mut c_char,
    _user_info: *const *mut c_char,
    _command_info: *const *mut c_char,
    _argc: c_int,
    _argv: *const *mut c_char,
    _user_env: *const *mut c_char,
    _plugin_plugins: *const *mut c_char
) -> c_int {
    return -2;
}
