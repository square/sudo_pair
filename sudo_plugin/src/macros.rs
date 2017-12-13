#[macro_export]
macro_rules! sudo_io_plugin {
    ( $name:ident : $ty:ty { $($cb:ident : $fn:ident),* $(,)* } ) => {
        use sudo_plugin::errors::AsSudoPluginRetval;

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
            $plugin = Some(sudo_plugin::Plugin::new(
                version,
                conversation,
                plugin_printf,
                settings_ptr,
                user_info_ptr,
                command_info_ptr,
                user_env_ptr,
                plugin_options_ptr,
            ));

            let plugin = $plugin.as_ref().unwrap();
            let instance = <$ty>::$fn(plugin);

            match instance {
                Ok(i)  => { $instance = Some(i) },
                Err(e) => { let _ = plugin.print_error(
                    format!("{}: {}\n", stringify!($name), e)
                ); },
            };

            match $instance {
                Some(_) =>  1,
                None    => -1,
            }
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
                    p.print_error(format!("{}: {}\n", stringify!($name), err))
                })
            });

            result.as_sudo_plugin_retval()
        }

        Some($log_fn)
    }};
}
