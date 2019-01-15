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
/// use sudo_plugin::*;
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
///     plugin: &'static sudo_plugin::Plugin
/// }
///
/// impl Example {
///     fn open(plugin: &'static sudo_plugin::Plugin) -> Result<Self> {
///         plugin.stdout().write(b"example sudo plugin initialized");
///
///         Ok(Example { plugin })
///     }
///
///     fn close(&mut self, _: i32, _: i32) {
///         self.plugin.stdout().write(b"example sudo plugin exited");
///     }
///
///     fn log_stdout(&mut self, _: &[u8]) -> Result<()> {
///         self.plugin.stdout().write(
///             b"example sudo plugin received output on stdout"
///         );
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
        use ::sudo_plugin::errors::AsSudoPluginRetval;

        static mut PLUGIN:   Option<::sudo_plugin::Plugin> = None;
        static mut INSTANCE: Option<$ty>                   = None;

        const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

        #[no_mangle]
        #[allow(non_upper_case_globals)]
        #[allow(missing_docs)]
        pub static $name: ::sudo_plugin::sys::io_plugin = {
            ::sudo_plugin::sys::io_plugin {
                // construct the plugin using any callbacks specified
                $( $cb: sudo_io_fn!($cb, $name, PLUGIN, INSTANCE, $fn) ),*,

                // and for anything not specified, use the defaults
                .. ::sudo_plugin::sys::io_plugin {
                    type_:            ::sudo_plugin::sys::SUDO_IO_PLUGIN,
                    version:          ::sudo_plugin::sys::SUDO_API_VERSION,
                    open:             Some(open),
                    close:            None,
                    show_version:     Some(show_version),
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

        unsafe extern "C" fn open(
            version:            ::libc::c_uint,
            conversation:       ::sudo_plugin::sys::sudo_conv_t,
            plugin_printf:      ::sudo_plugin::sys::sudo_printf_t,
            settings_ptr:       *const *mut ::libc::c_char,
            user_info_ptr:      *const *mut ::libc::c_char,
            command_info_ptr:   *const *mut ::libc::c_char,
            argc:               ::libc::c_int,
            argv:               *const *mut ::libc::c_char,
            user_env_ptr:       *const *mut ::libc::c_char,
            plugin_options_ptr: *const *mut ::libc::c_char,
        ) -> ::libc::c_int {
            unsafe fn stderr(
                printf: ::sudo_plugin::sys::sudo_printf_t,
                error:  &::sudo_plugin::errors::Error,
            ) -> ::libc::c_int {
                // if printf is a NULL pointer or if the `write_error`
                // call fails, we don't really have anything productive
                // to do
                if let Some(printf) = printf {
                    let printf = ::std::sync::Arc::new(
                        ::std::sync::Mutex::new(printf)
                    );

                    let _ = ::sudo_plugin::Printf {
                        facility: printf,
                        level:    ::sudo_plugin::sys::SUDO_CONV_ERROR_MSG,
                    }.write_error(stringify!($name).as_bytes(), error);
                }

                return error.as_sudo_io_plugin_open_retval();
            }

            let plugin = ::sudo_plugin::Plugin::new(
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
                Ok(p)  => PLUGIN = Some(p),
                Err(e) => return stderr(plugin_printf, &e),
            }

            // unwrap should be panic-safe here, since we just assigned
            // a value to $plugin
            let plugin = PLUGIN.as_ref().unwrap();

            // if the command is empty, to the best of my knowledge
            // we're being called with `-V` to report our version; in
            // this case there's no reason to fully the plugin through
            // its `open` function
            if plugin.command_info.command == ::std::path::PathBuf::default() {
                return ::sudo_plugin::sys::SUDO_PLUGIN_OPEN_SUCCESS;
            }

            // call the plugin's `open` function
            match <$ty>::open(plugin) {
                Ok(i)  => INSTANCE = Some(i),
                Err(e) => return stderr(plugin_printf, &e.into()),
            }

            ::sudo_plugin::sys::SUDO_PLUGIN_OPEN_SUCCESS
        }

        unsafe extern "C" fn show_version(
            _verbose: ::libc::c_int,
        ) -> ::libc::c_int {
            if let Some(plugin) = PLUGIN.as_ref() {
                // disable the write_literal lint since it has a known
                // bug that fires when you use a macro that expands to
                // a literal (e.g., `stringify!`)
                #[cfg_attr(feature="cargo-clippy", allow(clippy::write_literal))]
                let _ = writeln!(plugin.stdout(),
                    "{} I/O plugin version {}",
                    stringify!($name), VERSION.unwrap_or("unknown")
                );
            }

            0
        }
    }
}

/// Internal macro used by `sudo_io_plugin` that  generates the actual
/// callback implementations for I/O plugins.
#[macro_export]
macro_rules! sudo_io_fn {
    ( close , $name:tt , $plugin:expr , $instance:expr , $fn:ident ) => {{
        unsafe extern "C" fn close(
            exit_status: ::libc::c_int,
            error:       ::libc::c_int,
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
            len:        ::libc::c_uint,
        ) -> ::libc::c_int {
            let slice = ::std::slice::from_raw_parts(
                buf as *const _,
                len as _,
            );

            let result : ::std::result::Result<(), ::sudo_plugin::errors::Error> = $instance
                .as_mut()
                .map_or_else(
                  || Err(::sudo_plugin::errors::ErrorKind::Uninitialized.into()),
                  |i| i.$fn(slice).map_err(|e| e.into()),
                );

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
        ) {
            let result : ::std::result::Result<(), ::sudo_plugin::errors::Error> = $instance
                .as_mut()
                .map_or_else(
                  || Err(::sudo_plugin::errors::ErrorKind::Uninitialized.into()),
                  |i| i.$fn(slice).map_err(|e| e.into()),
                );

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
