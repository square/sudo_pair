//! Macros to simplify the process of correctly wiring up a sudo plugin.

/// Emits the boilerplate stanza for creating and initializing a custom
/// sudo I/O plugin.
///
/// # Example
///
/// ```rust
/// # #[macro_use] extern crate sudo_plugin;
///
/// sudo_io_plugin! {
///     example : Example {
///         close:      close,
///         log_stdout: log_stdout,
///     }
/// }
///
/// struct Example {
/// }
///
/// impl Example {
///     fn open(plugin: &'static sudo_plugin::Plugin) -> Result<Self> {
///         plugin.print_info("example sudo plugin initialized");
///
///         Ok(Example {})
///     }
///
///     fn close(&mut self, _: i32, _: i32) {
///         plugin.print_info("example sudo plugin exited");
///     }
///
///     fn log_stdout(&mut self, _: &[u8]) -> Result<()> {
///         plugin.print_info("example sudo plugin received output on stdout");
///     }
/// }
/// ```
///
/// The generated plugin will have the entry point `example`, so to
/// enable it, you'd copy the library to `example.so` in sudo's plugin
/// directory (on macOS, `/usr/local/libexec/sudo`) and add the following
/// to `/etc/sudo.conf`:
///
/// ```
/// Plugin example example.so
/// ```
#[macro_export]
macro_rules! sudo_io_plugin {
    ( $name:ident : $ty:ty { $($cb:ident : $fn:ident),* $(,)* } ) => {
        use sudo_plugin::errors::*;

        static mut PLUGIN:   Option<sudo_plugin::Plugin> = None;
        static mut INSTANCE: Option<$ty>                 = None;

        #[no_mangle]
        #[allow(non_upper_case_globals)]
        #[allow(missing_docs)]
        pub static $name: sudo_plugin::sys::io_plugin = {
            sudo_plugin::sys::io_plugin {
                open: sudo_io_static_fn!(open, $name, PLUGIN, INSTANCE, $ty, open),

                $( $cb: sudo_io_fn!($cb, $name, PLUGIN, INSTANCE, $fn) ),*,

                .. sudo_plugin::sys::io_plugin {
                    type_:            sudo_plugin::sys::SUDO_IO_PLUGIN,
                    version:          sudo_plugin::sys::SUDO_API_VERSION,
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
                }
            }
        };
    }
}

#[macro_export]
macro_rules! sudo_io_static_fn {
    ( open , $name:tt , $plugin:expr , $instance:expr , $ty:ty , $fn:ident ) => {{
        unsafe extern "C" fn sudo_plugin_open(
            version:            c_uint,
            conversation:       sudo_plugin::sys::sudo_conv_t,
            plugin_printf:      sudo_plugin::sys::sudo_printf_t,
            settings_ptr:       *const *const c_char,
            user_info_ptr:      *const *const c_char,
            command_info_ptr:   *const *const c_char,
            _argc:              c_int,
            _argv:              *const *const c_char,
            user_env_ptr:       *const *const c_char,
            plugin_options_ptr: *const *const c_char,
        ) -> c_int {
            let plugin = sudo_plugin::Plugin::new(
                version,
                conversation,
                plugin_printf,
                settings_ptr,
                user_info_ptr,
                command_info_ptr,
                user_env_ptr,
                plugin_options_ptr,
            );

            let instance = match plugin {
                Ok(p)  => {
                    $plugin = Some(p);
                    <$ty>::$fn($plugin.as_ref().unwrap())
                }

                Err(e) => Err(e),
            };

            let ret = instance.as_sudo_plugin_retval();

            match instance {
                Ok(i) => {
                    $instance = Some(i);
                },

                Err(e) => {
                    let _ = sudo_plugin::Plugin::printf(
                        plugin_printf,
                        3,
                        e.description(),
                    );
                }
            }

            ret
        }

        Some(sudo_plugin_open)
    }};
}

#[macro_export]
macro_rules! sudo_io_fn {
    ( close , $name:tt , $plugin:expr , $instance:expr , $fn:ident ) => {{
        unsafe extern "C" fn close(
            exit_status: c_int,
            error: c_int
        ) {
            if let Some(ref mut i) = $instance {
                i.$fn(exit_status, error)
            }
        }

        Some(close)
    }};

    ( log_ttyin , $name:tt, $plugin:expr , $instance:expr , $fn:ident ) => {
        sudo_io_fn!(log, log_ttyin, $name, $plugin, $instance, $fn)
    };

    ( log_ttyout , $name:tt, $plugin:expr , $instance:expr , $fn:ident ) => {
        sudo_io_fn!(log, log_ttyout, $name, $plugin, $instance, $fn)
    };

    ( log_stdin , $name:tt, $plugin:expr , $instance:expr , $fn:ident ) => {
        sudo_io_fn!(log, log_stdin, $name, $plugin, $instance, $fn)
    };

    ( log_stdout , $name:tt, $plugin:expr , $instance:expr , $fn:ident ) => {
        sudo_io_fn!(log, log_stdout, $name, $plugin, $instance, $fn)
    };

    ( log_stderr , $name:tt, $plugin:expr , $instance:expr , $fn:ident ) => {
        sudo_io_fn!(log, log_stderr, $name, $plugin, $instance, $fn)
    };

    ( log , $log_fn:ident , $name:tt , $plugin:expr , $instance:expr , $fn:ident ) => {{
        unsafe extern "C" fn $log_fn(
            buf: *const c_char,
            len: c_uint,
        ) -> c_int {
            let slice = std::slice::from_raw_parts(
                buf as *const _,
                len as _,
            );

            let result = $instance
                .as_mut()
                .ok_or(ErrorKind::Uninitialized.into())
                .and_then(|i| i.$fn(slice) );

            let _ = result.as_ref().map_err(|err| {
                $plugin.as_ref().map(|p| {
                    p.print_error(&format!("{}: {}\n", stringify!($name), err))
                })
            });

            result.as_sudo_plugin_retval()
        }

        Some($log_fn)
    }};
}
