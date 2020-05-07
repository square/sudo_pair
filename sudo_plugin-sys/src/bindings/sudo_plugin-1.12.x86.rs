/* automatically generated by rust-bindgen */

pub const SUDO_API_VERSION_MAJOR: u32 = 1;
pub const SUDO_API_VERSION_MINOR: u32 = 12;
pub const SUDO_CONV_PROMPT_ECHO_OFF: u32 = 1;
pub const SUDO_CONV_PROMPT_ECHO_ON: u32 = 2;
pub const SUDO_CONV_ERROR_MSG: u32 = 3;
pub const SUDO_CONV_INFO_MSG: u32 = 4;
pub const SUDO_CONV_PROMPT_MASK: u32 = 5;
pub const SUDO_CONV_PROMPT_ECHO_OK: u32 = 4096;
pub const SUDO_CONV_REPL_MAX: u32 = 255;
pub const SUDO_CONV_CALLBACK_VERSION_MAJOR: u32 = 1;
pub const SUDO_CONV_CALLBACK_VERSION_MINOR: u32 = 0;
pub const SUDO_HOOK_VERSION_MAJOR: u32 = 1;
pub const SUDO_HOOK_VERSION_MINOR: u32 = 0;
pub const SUDO_HOOK_RET_ERROR: i32 = -1;
pub const SUDO_HOOK_RET_NEXT: u32 = 0;
pub const SUDO_HOOK_RET_STOP: u32 = 1;
pub const SUDO_HOOK_SETENV: u32 = 1;
pub const SUDO_HOOK_UNSETENV: u32 = 2;
pub const SUDO_HOOK_PUTENV: u32 = 3;
pub const SUDO_HOOK_GETENV: u32 = 4;
pub const SUDO_POLICY_PLUGIN: u32 = 1;
pub const SUDO_IO_PLUGIN: u32 = 2;
pub const GROUP_API_VERSION_MAJOR: u32 = 1;
pub const GROUP_API_VERSION_MINOR: u32 = 0;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sudo_conv_message {
    pub msg_type: ::std::os::raw::c_int,
    pub timeout: ::std::os::raw::c_int,
    pub msg: *const ::std::os::raw::c_char,
}
#[test]
fn bindgen_test_layout_sudo_conv_message() {
    assert_eq!(
        ::std::mem::size_of::<sudo_conv_message>(),
        12usize,
        concat!("Size of: ", stringify!(sudo_conv_message))
    );
    assert_eq!(
        ::std::mem::align_of::<sudo_conv_message>(),
        4usize,
        concat!("Alignment of ", stringify!(sudo_conv_message))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudo_conv_message>())).msg_type as *const _
                as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(sudo_conv_message),
            "::",
            stringify!(msg_type)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudo_conv_message>())).timeout as *const _
                as usize
        },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(sudo_conv_message),
            "::",
            stringify!(timeout)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudo_conv_message>())).msg as *const _
                as usize
        },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(sudo_conv_message),
            "::",
            stringify!(msg)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sudo_conv_reply {
    pub reply: *mut ::std::os::raw::c_char,
}
#[test]
fn bindgen_test_layout_sudo_conv_reply() {
    assert_eq!(
        ::std::mem::size_of::<sudo_conv_reply>(),
        4usize,
        concat!("Size of: ", stringify!(sudo_conv_reply))
    );
    assert_eq!(
        ::std::mem::align_of::<sudo_conv_reply>(),
        4usize,
        concat!("Alignment of ", stringify!(sudo_conv_reply))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudo_conv_reply>())).reply as *const _
                as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(sudo_conv_reply),
            "::",
            stringify!(reply)
        )
    );
}
pub type sudo_conv_callback_fn_t = ::std::option::Option<
    unsafe extern "C" fn(
        signo: ::std::os::raw::c_int,
        closure: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int,
>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sudo_conv_callback {
    pub version: ::std::os::raw::c_uint,
    pub closure: *mut ::std::os::raw::c_void,
    pub on_suspend: sudo_conv_callback_fn_t,
    pub on_resume: sudo_conv_callback_fn_t,
}
#[test]
fn bindgen_test_layout_sudo_conv_callback() {
    assert_eq!(
        ::std::mem::size_of::<sudo_conv_callback>(),
        16usize,
        concat!("Size of: ", stringify!(sudo_conv_callback))
    );
    assert_eq!(
        ::std::mem::align_of::<sudo_conv_callback>(),
        4usize,
        concat!("Alignment of ", stringify!(sudo_conv_callback))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudo_conv_callback>())).version as *const _
                as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(sudo_conv_callback),
            "::",
            stringify!(version)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudo_conv_callback>())).closure as *const _
                as usize
        },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(sudo_conv_callback),
            "::",
            stringify!(closure)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudo_conv_callback>())).on_suspend
                as *const _ as usize
        },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(sudo_conv_callback),
            "::",
            stringify!(on_suspend)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudo_conv_callback>())).on_resume as *const _
                as usize
        },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(sudo_conv_callback),
            "::",
            stringify!(on_resume)
        )
    );
}
pub type sudo_conv_t = ::std::option::Option<
    unsafe extern "C" fn(
        num_msgs: ::std::os::raw::c_int,
        msgs: *const sudo_conv_message,
        replies: *mut sudo_conv_reply,
        callback: *mut sudo_conv_callback,
    ) -> ::std::os::raw::c_int,
>;
pub type sudo_printf_t = ::std::option::Option<
    unsafe extern "C" fn(
        msg_type: ::std::os::raw::c_int,
        fmt: *const ::std::os::raw::c_char,
        ...
    ) -> ::std::os::raw::c_int,
>;
pub type sudo_hook_fn_t =
    ::std::option::Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>;
pub type sudo_hook_fn_setenv_t = ::std::option::Option<
    unsafe extern "C" fn(
        name: *const ::std::os::raw::c_char,
        value: *const ::std::os::raw::c_char,
        overwrite: ::std::os::raw::c_int,
        closure: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int,
>;
pub type sudo_hook_fn_putenv_t = ::std::option::Option<
    unsafe extern "C" fn(
        string: *mut ::std::os::raw::c_char,
        closure: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int,
>;
pub type sudo_hook_fn_getenv_t = ::std::option::Option<
    unsafe extern "C" fn(
        name: *const ::std::os::raw::c_char,
        value: *mut *mut ::std::os::raw::c_char,
        closure: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int,
>;
pub type sudo_hook_fn_unsetenv_t = ::std::option::Option<
    unsafe extern "C" fn(
        name: *const ::std::os::raw::c_char,
        closure: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int,
>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sudo_hook {
    pub hook_version: ::std::os::raw::c_uint,
    pub hook_type: ::std::os::raw::c_uint,
    pub hook_fn: sudo_hook_fn_t,
    pub closure: *mut ::std::os::raw::c_void,
}
#[test]
fn bindgen_test_layout_sudo_hook() {
    assert_eq!(
        ::std::mem::size_of::<sudo_hook>(),
        16usize,
        concat!("Size of: ", stringify!(sudo_hook))
    );
    assert_eq!(
        ::std::mem::align_of::<sudo_hook>(),
        4usize,
        concat!("Alignment of ", stringify!(sudo_hook))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudo_hook>())).hook_version as *const _
                as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(sudo_hook),
            "::",
            stringify!(hook_version)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudo_hook>())).hook_type as *const _ as usize
        },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(sudo_hook),
            "::",
            stringify!(hook_type)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudo_hook>())).hook_fn as *const _ as usize
        },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(sudo_hook),
            "::",
            stringify!(hook_fn)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudo_hook>())).closure as *const _ as usize
        },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(sudo_hook),
            "::",
            stringify!(closure)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct passwd {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct policy_plugin {
    pub type_: ::std::os::raw::c_uint,
    pub version: ::std::os::raw::c_uint,
    pub open: ::std::option::Option<
        unsafe extern "C" fn(
            version: ::std::os::raw::c_uint,
            conversation: sudo_conv_t,
            sudo_printf: sudo_printf_t,
            settings: *const *mut ::std::os::raw::c_char,
            user_info: *const *mut ::std::os::raw::c_char,
            user_env: *const *mut ::std::os::raw::c_char,
            plugin_plugins: *const *mut ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_int,
    >,
    pub close: ::std::option::Option<
        unsafe extern "C" fn(
            exit_status: ::std::os::raw::c_int,
            error: ::std::os::raw::c_int,
        ),
    >,
    pub show_version: ::std::option::Option<
        unsafe extern "C" fn(
            verbose: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub check_policy: ::std::option::Option<
        unsafe extern "C" fn(
            argc: ::std::os::raw::c_int,
            argv: *const *mut ::std::os::raw::c_char,
            env_add: *mut *mut ::std::os::raw::c_char,
            command_info: *mut *mut *mut ::std::os::raw::c_char,
            argv_out: *mut *mut *mut ::std::os::raw::c_char,
            user_env_out: *mut *mut *mut ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_int,
    >,
    pub list: ::std::option::Option<
        unsafe extern "C" fn(
            argc: ::std::os::raw::c_int,
            argv: *const *mut ::std::os::raw::c_char,
            verbose: ::std::os::raw::c_int,
            list_user: *const ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_int,
    >,
    pub validate:
        ::std::option::Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub invalidate: ::std::option::Option<
        unsafe extern "C" fn(remove: ::std::os::raw::c_int),
    >,
    pub init_session: ::std::option::Option<
        unsafe extern "C" fn(
            pwd: *mut passwd,
            user_env_out: *mut *mut *mut ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_int,
    >,
    pub register_hooks: ::std::option::Option<
        unsafe extern "C" fn(
            version: ::std::os::raw::c_int,
            register_hook: ::std::option::Option<
                unsafe extern "C" fn(
                    hook: *mut sudo_hook,
                ) -> ::std::os::raw::c_int,
            >,
        ),
    >,
    pub deregister_hooks: ::std::option::Option<
        unsafe extern "C" fn(
            version: ::std::os::raw::c_int,
            deregister_hook: ::std::option::Option<
                unsafe extern "C" fn(
                    hook: *mut sudo_hook,
                ) -> ::std::os::raw::c_int,
            >,
        ),
    >,
}
#[test]
fn bindgen_test_layout_policy_plugin() {
    assert_eq!(
        ::std::mem::size_of::<policy_plugin>(),
        48usize,
        concat!("Size of: ", stringify!(policy_plugin))
    );
    assert_eq!(
        ::std::mem::align_of::<policy_plugin>(),
        4usize,
        concat!("Alignment of ", stringify!(policy_plugin))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<policy_plugin>())).type_ as *const _ as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(policy_plugin),
            "::",
            stringify!(type_)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<policy_plugin>())).version as *const _
                as usize
        },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(policy_plugin),
            "::",
            stringify!(version)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<policy_plugin>())).open as *const _ as usize
        },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(policy_plugin),
            "::",
            stringify!(open)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<policy_plugin>())).close as *const _ as usize
        },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(policy_plugin),
            "::",
            stringify!(close)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<policy_plugin>())).show_version as *const _
                as usize
        },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(policy_plugin),
            "::",
            stringify!(show_version)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<policy_plugin>())).check_policy as *const _
                as usize
        },
        20usize,
        concat!(
            "Offset of field: ",
            stringify!(policy_plugin),
            "::",
            stringify!(check_policy)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<policy_plugin>())).list as *const _ as usize
        },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(policy_plugin),
            "::",
            stringify!(list)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<policy_plugin>())).validate as *const _
                as usize
        },
        28usize,
        concat!(
            "Offset of field: ",
            stringify!(policy_plugin),
            "::",
            stringify!(validate)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<policy_plugin>())).invalidate as *const _
                as usize
        },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(policy_plugin),
            "::",
            stringify!(invalidate)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<policy_plugin>())).init_session as *const _
                as usize
        },
        36usize,
        concat!(
            "Offset of field: ",
            stringify!(policy_plugin),
            "::",
            stringify!(init_session)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<policy_plugin>())).register_hooks as *const _
                as usize
        },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(policy_plugin),
            "::",
            stringify!(register_hooks)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<policy_plugin>())).deregister_hooks
                as *const _ as usize
        },
        44usize,
        concat!(
            "Offset of field: ",
            stringify!(policy_plugin),
            "::",
            stringify!(deregister_hooks)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct io_plugin {
    pub type_: ::std::os::raw::c_uint,
    pub version: ::std::os::raw::c_uint,
    pub open: ::std::option::Option<
        unsafe extern "C" fn(
            version: ::std::os::raw::c_uint,
            conversation: sudo_conv_t,
            sudo_printf: sudo_printf_t,
            settings: *const *mut ::std::os::raw::c_char,
            user_info: *const *mut ::std::os::raw::c_char,
            command_info: *const *mut ::std::os::raw::c_char,
            argc: ::std::os::raw::c_int,
            argv: *const *mut ::std::os::raw::c_char,
            user_env: *const *mut ::std::os::raw::c_char,
            plugin_plugins: *const *mut ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_int,
    >,
    pub close: ::std::option::Option<
        unsafe extern "C" fn(
            exit_status: ::std::os::raw::c_int,
            error: ::std::os::raw::c_int,
        ),
    >,
    pub show_version: ::std::option::Option<
        unsafe extern "C" fn(
            verbose: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub log_ttyin: ::std::option::Option<
        unsafe extern "C" fn(
            buf: *const ::std::os::raw::c_char,
            len: ::std::os::raw::c_uint,
        ) -> ::std::os::raw::c_int,
    >,
    pub log_ttyout: ::std::option::Option<
        unsafe extern "C" fn(
            buf: *const ::std::os::raw::c_char,
            len: ::std::os::raw::c_uint,
        ) -> ::std::os::raw::c_int,
    >,
    pub log_stdin: ::std::option::Option<
        unsafe extern "C" fn(
            buf: *const ::std::os::raw::c_char,
            len: ::std::os::raw::c_uint,
        ) -> ::std::os::raw::c_int,
    >,
    pub log_stdout: ::std::option::Option<
        unsafe extern "C" fn(
            buf: *const ::std::os::raw::c_char,
            len: ::std::os::raw::c_uint,
        ) -> ::std::os::raw::c_int,
    >,
    pub log_stderr: ::std::option::Option<
        unsafe extern "C" fn(
            buf: *const ::std::os::raw::c_char,
            len: ::std::os::raw::c_uint,
        ) -> ::std::os::raw::c_int,
    >,
    pub register_hooks: ::std::option::Option<
        unsafe extern "C" fn(
            version: ::std::os::raw::c_int,
            register_hook: ::std::option::Option<
                unsafe extern "C" fn(
                    hook: *mut sudo_hook,
                ) -> ::std::os::raw::c_int,
            >,
        ),
    >,
    pub deregister_hooks: ::std::option::Option<
        unsafe extern "C" fn(
            version: ::std::os::raw::c_int,
            deregister_hook: ::std::option::Option<
                unsafe extern "C" fn(
                    hook: *mut sudo_hook,
                ) -> ::std::os::raw::c_int,
            >,
        ),
    >,
    pub change_winsize: ::std::option::Option<
        unsafe extern "C" fn(
            rows: ::std::os::raw::c_uint,
            cols: ::std::os::raw::c_uint,
        ) -> ::std::os::raw::c_int,
    >,
}
#[test]
fn bindgen_test_layout_io_plugin() {
    assert_eq!(
        ::std::mem::size_of::<io_plugin>(),
        52usize,
        concat!("Size of: ", stringify!(io_plugin))
    );
    assert_eq!(
        ::std::mem::align_of::<io_plugin>(),
        4usize,
        concat!("Alignment of ", stringify!(io_plugin))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<io_plugin>())).type_ as *const _ as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(io_plugin),
            "::",
            stringify!(type_)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<io_plugin>())).version as *const _ as usize
        },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(io_plugin),
            "::",
            stringify!(version)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<io_plugin>())).open as *const _ as usize
        },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(io_plugin),
            "::",
            stringify!(open)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<io_plugin>())).close as *const _ as usize
        },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(io_plugin),
            "::",
            stringify!(close)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<io_plugin>())).show_version as *const _
                as usize
        },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(io_plugin),
            "::",
            stringify!(show_version)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<io_plugin>())).log_ttyin as *const _ as usize
        },
        20usize,
        concat!(
            "Offset of field: ",
            stringify!(io_plugin),
            "::",
            stringify!(log_ttyin)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<io_plugin>())).log_ttyout as *const _
                as usize
        },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(io_plugin),
            "::",
            stringify!(log_ttyout)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<io_plugin>())).log_stdin as *const _ as usize
        },
        28usize,
        concat!(
            "Offset of field: ",
            stringify!(io_plugin),
            "::",
            stringify!(log_stdin)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<io_plugin>())).log_stdout as *const _
                as usize
        },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(io_plugin),
            "::",
            stringify!(log_stdout)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<io_plugin>())).log_stderr as *const _
                as usize
        },
        36usize,
        concat!(
            "Offset of field: ",
            stringify!(io_plugin),
            "::",
            stringify!(log_stderr)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<io_plugin>())).register_hooks as *const _
                as usize
        },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(io_plugin),
            "::",
            stringify!(register_hooks)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<io_plugin>())).deregister_hooks as *const _
                as usize
        },
        44usize,
        concat!(
            "Offset of field: ",
            stringify!(io_plugin),
            "::",
            stringify!(deregister_hooks)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<io_plugin>())).change_winsize as *const _
                as usize
        },
        48usize,
        concat!(
            "Offset of field: ",
            stringify!(io_plugin),
            "::",
            stringify!(change_winsize)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sudoers_group_plugin {
    pub version: ::std::os::raw::c_uint,
    pub init: ::std::option::Option<
        unsafe extern "C" fn(
            version: ::std::os::raw::c_int,
            sudo_printf: sudo_printf_t,
            argv: *const *mut ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_int,
    >,
    pub cleanup: ::std::option::Option<unsafe extern "C" fn()>,
    pub query: ::std::option::Option<
        unsafe extern "C" fn(
            user: *const ::std::os::raw::c_char,
            group: *const ::std::os::raw::c_char,
            pwd: *const passwd,
        ) -> ::std::os::raw::c_int,
    >,
}
#[test]
fn bindgen_test_layout_sudoers_group_plugin() {
    assert_eq!(
        ::std::mem::size_of::<sudoers_group_plugin>(),
        16usize,
        concat!("Size of: ", stringify!(sudoers_group_plugin))
    );
    assert_eq!(
        ::std::mem::align_of::<sudoers_group_plugin>(),
        4usize,
        concat!("Alignment of ", stringify!(sudoers_group_plugin))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudoers_group_plugin>())).version as *const _
                as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(sudoers_group_plugin),
            "::",
            stringify!(version)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudoers_group_plugin>())).init as *const _
                as usize
        },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(sudoers_group_plugin),
            "::",
            stringify!(init)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudoers_group_plugin>())).cleanup as *const _
                as usize
        },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(sudoers_group_plugin),
            "::",
            stringify!(cleanup)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<sudoers_group_plugin>())).query as *const _
                as usize
        },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(sudoers_group_plugin),
            "::",
            stringify!(query)
        )
    );
}
