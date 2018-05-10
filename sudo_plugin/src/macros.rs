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

//! Macros to simplify the process of correctly wiring up a sudo plugin.

/// Emits the boilerplate stanza for creating and initializing a custom
/// sudo I/O plugin.
///
/// # Example
///
/// ```rust
/// # #[macro_use] extern crate sudo_plugin;
/// extern crate libc;
///
/// # fn main() { } // TODO(rust 1.27): remove this line
///
/// use sudo_plugin::errors::*;
/// use std::io::Write;
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
///         plugin.stdout().write(b"example sudo plugin initialized");
///
///         Ok(Example {})
///     }
///
///     fn close(&mut self, _: i32, _: i32) {
///         println!("example sudo plugin exited");
///     }
///
///     fn log_stdout(&mut self, _: &[u8]) -> Result<()> {
///         println!("example sudo plugin received output on stdout");
///
///         Ok(())
///     }
/// }
/// ```
///
/// The generated plugin will have the entry point `example`, so to
/// enable it, you'd copy the library to `example.so` in sudo's plugin
/// directory (on macOS, `/usr/local/libexec/sudo`) and add the following
/// to `/etc/sudo.conf`:
///
/// ```ignore
/// Plugin example example.so
/// ```
#[macro_export]
macro_rules! sudo_io_plugin {
    ( $name:ident : $ty:ty { $($cb:ident : $fn:ident),* $(,)* } ) => {
        static mut PLUGIN:   Option<sudo_plugin::Plugin> = None;
        static mut INSTANCE: Option<$ty>                 = None;

        #[no_mangle]
        #[allow(non_upper_case_globals)]
        #[allow(missing_docs)]
        pub static $name: sudo_plugin::sys::io_plugin = {
            sudo_plugin::sys::io_plugin {
                open: sudo_io_static_fn!(
                    open, $name, PLUGIN, INSTANCE, $ty, open
                ),

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
                    change_winsize:   None,
                }
            }
        };
    }
}

#[macro_export]
macro_rules! sudo_io_static_fn {
    (
        open ,
        $name:tt ,
        $plugin:expr ,
        $instance:expr ,
        $ty:ty ,
        $fn:ident
    ) => {{
        unsafe extern "C" fn open(
            version:            ::libc::c_uint,
            conversation:       sudo_plugin::sys::sudo_conv_t,
            plugin_printf:      sudo_plugin::sys::sudo_printf_t,
            settings_ptr:       *const *const ::libc::c_char,
            user_info_ptr:      *const *const ::libc::c_char,
            command_info_ptr:   *const *const ::libc::c_char,
            argc:               ::libc::c_int,
            argv:               *const *const ::libc::c_char,
            user_env_ptr:       *const *const ::libc::c_char,
            plugin_options_ptr: *const *const ::libc::c_char,
        ) -> ::libc::c_int {
            unsafe fn stderr(
                printf: sudo_plugin::sys::sudo_printf_t,
                error:  &Error,
            ) -> ::libc::c_int {
                // if printf is a NULL pointer or if the `write_error`
                // call fails, we don't really have anything productive
                // to do
                if let Some(printf) = printf {
                    let printf = ::std::sync::Arc::new(
                        ::std::sync::Mutex::new(printf)
                    );

                    let _ = sudo_plugin::Printf {
                        facility: printf,
                        level:    sudo_plugin::sys::SUDO_CONV_ERROR_MSG,
                    }.write_error(stringify!($name).as_bytes(), error);
                }

                return error.as_sudo_io_plugin_open_retval();
            }

            let plugin = sudo_plugin::Plugin::new(
                version,
                argc, argv,
                conversation,
                plugin_printf,
                settings_ptr,
                user_info_ptr,
                command_info_ptr,
                user_env_ptr,
                plugin_options_ptr,
            );

            match plugin {
                Ok(p)  => $plugin = Some(p),
                Err(e) => return stderr(plugin_printf, &e),
            }

            // unwrap should be panic-safe here, since we just assigned
            // a value to $plugin
            let instance = <$ty>::$fn($plugin.as_ref().unwrap());

            match instance {
                Ok(i)  => $instance = Some(i),
                Err(e) => return stderr(plugin_printf, &e),
            }

            sudo_plugin::sys::SUDO_PLUGIN_OPEN_SUCCESS
        }

        Some(open)
    }};
}

#[macro_export]
macro_rules! sudo_io_fn {
    ( close , $name:tt , $plugin:expr , $instance:expr , $fn:ident ) => {{
        unsafe extern "C" fn close(
            exit_status: ::libc::c_int,
            error:       ::libc::c_int
        ) {
            if let Some(i) = $instance.as_mut() {
                i.$fn(exit_status as _, error as _)
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

    (
        log ,
        $log_fn:ident ,
        $name:tt ,
        $plugin:expr ,
        $instance:expr ,
        $fn:ident
    ) => {{
        unsafe extern "C" fn $log_fn(
            buf: *const ::libc::c_char,
            len: ::libc::c_uint,
        ) -> ::libc::c_int {
            let slice = std::slice::from_raw_parts(
                buf as *const _,
                len as _,
            );

            let result = $instance
                .as_mut()
                .ok_or(::sudo_plugin::errors::ErrorKind::Uninitialized.into())
                .and_then(|i| i.$fn(slice) );

            // if there was an error (and we can unwrap the plugin),
            // write it out
            if let (Some(p), Err(e)) = ($plugin.as_ref(), result.as_ref()) {
                let _ = p.stderr().write_error(
                    stringify!($name).as_bytes(),
                    e,
                );
            }

            result.as_sudo_io_plugin_log_retval()
        }

        Some($log_fn)
    }};

    ( change_winsize , $name:tt , $plugin:expr , $instance:expr , $fn:ident ) => {{
        unsafe extern "C" fn change_winsize(
            lines: ::libc::c_uint,
            cols:  ::libc::c_uint,
        ) -> ::libc::c_int {
            let result = $instance
                .as_mut()
                .ok_or(::sudo_plugin::errors::ErrorKind::Uninitialized.into())
                .and_then(|i| i.$fn(lines as _, cols as _) );

            // if there was an error (and we can unwrap the plugin),
            // write it out
            if let (Some(p), Err(e)) = ($plugin.as_ref(), result.as_ref()) {
                let _ = p.stderr().write_error(
                    stringify!($name).as_bytes(),
                    e,
                );
            }

            result.as_sudo_io_plugin_log_retval()
        }

        Some(change_winsize)
    }};
}
