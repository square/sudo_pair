//! Rust bindings to sudo's plugin API.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(missing_copy_implementations)]
#![allow(missing_debug_implementations)]

// bitflags crate creates these warnings
#![allow(trivial_numeric_casts)]

use libc::{c_char, c_int, c_uint, c_void};

pub const SUDO_API_VERSION_MAJOR: c_uint = 1;
pub const SUDO_API_VERSION_MINOR: c_uint = 9;
pub const SUDO_API_VERSION:       c_uint
    = SUDO_API_VERSION_MAJOR << 16
    | SUDO_API_VERSION_MINOR;

pub enum SUDO_PLUGIN {
    #[allow(dead_code)]

    POLICY = 0x01, // policy plugin identifier
    IO     = 0x02, // io plugin identifier
}

bitflags! {
    pub flags SUDO_CONV_FLAGS: c_int {
        const SUDO_CONV_PROMPT_ECHO_OFF = 0x0001, // do not echo user input
        const SUDO_CONV_PROMPT_ECHO_ON  = 0x0002, // echo user input
        const SUDO_CONV_ERROR_MSG       = 0x0003, // error message
        const SUDO_CONV_INFO_MSG        = 0x0004, // informational message
        const SUDO_CONV_PROMPT_MASK     = 0x0005, // mask user input
        const SUDO_CONV_DEBUG_MSG       = 0x0006, // debugging message
        const SUDO_CONV_PROMPT_ECHO_OK  = 0x1000, // flag: allow echo if no tty
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[repr(C)]
pub struct io_plugin {
    pub type_:            c_uint,
    pub version:          c_uint,
    pub open:             Option<sudo_open_fn_t>,
    pub close:            Option<sudo_close_fn_t>,
    pub show_version:     Option<sudo_show_version_fn_t>,
    pub log_ttyin:        Option<sudo_log_fn_t>,
    pub log_ttyout:       Option<sudo_log_fn_t>,
    pub log_stdin:        Option<sudo_log_fn_t>,
    pub log_stdout:       Option<sudo_log_fn_t>,
    pub log_stderr:       Option<sudo_log_fn_t>,
    pub register_hooks:   Option<sudo_hook_registration_fn_t>,
    pub deregister_hooks: Option<sudo_hook_registration_fn_t>,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[repr(C)]
pub struct sudo_hook {
    pub hook_version: c_uint,
    pub hook_type:    c_uint,
    pub hook_fn:      sudo_hook_fn_t,
    pub closure:      *mut c_void,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[repr(C)]
pub struct sudo_conv_message {
    pub msg_type: c_int,
    pub timeout:  c_int,
    pub msg:      *const c_char,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[repr(C)]
pub struct sudo_conv_reply {
    pub reply: *mut c_char,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[repr(C)]
pub struct sudo_conv_callback {
    pub version:    c_uint,
    pub closure:    *mut c_void,
    pub on_suspend: sudo_conv_callback_fn_t,
    pub on_resume:  sudo_conv_callback_fn_t,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type sudo_open_fn_t = unsafe extern "C" fn(
    version:        c_uint,
    conversation:   sudo_conv_t,
    sudo_printf:    sudo_printf_t,
    settings:       *const *mut c_char,
    user_info:      *const *mut c_char,
    command_info:   *const *mut c_char,
    argc:           c_int,
    argv:           *const *mut c_char,
    user_env:       *const *mut c_char,
    plugin_options: *const *mut c_char,
) -> c_int;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type sudo_close_fn_t = unsafe extern "C" fn(
    exit_status: c_int,
    error:       c_int,
);

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type sudo_show_version_fn_t = unsafe extern "C" fn(
    verbose: c_int,
) -> c_int;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type sudo_log_fn_t = unsafe extern "C" fn(
    buf: *const c_char,
    len: c_uint,
) -> c_int;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type sudo_hook_registration_fn_t = unsafe extern "C" fn(
    version:       c_int,
    register_hook: sudo_hook_register_fn_t,
);

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type sudo_hook_register_fn_t = unsafe extern "C" fn(
    hook: *mut sudo_hook,
) -> c_int;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type sudo_conv_t = unsafe extern "C" fn(
    num_msgs: c_int,
    msgs:     *mut sudo_conv_message,
    replies:  *mut sudo_conv_reply,
    callback: *mut sudo_conv_callback,
) -> c_int;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type sudo_printf_t = unsafe extern "C" fn(
    msg_type: c_int,
    fmt:      *const c_char,
    ...
) -> c_int;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type sudo_hook_fn_t = unsafe extern "C" fn() -> c_int;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type sudo_conv_callback_fn_t = unsafe extern "C" fn(
    signo:   c_int,
    closure: *mut c_void,
) -> c_int;
