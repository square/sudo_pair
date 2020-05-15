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
/// # mod necessary_for_super_type_lookup_to_work {
/// use sudo_plugin::*;
/// use sudo_plugin::errors::*;
/// use std::io::Write;
///
/// sudo_io_plugin! { example : Example }
///
/// struct Example {
///     env: &'static sudo_plugin::IoEnv
/// }
///
/// impl IoPlugin for Example {
///     const NAME:    &'static str = "example";
///     const VERSION: &'static str = env!("CARGO_PKG_VERSION");
///
///     fn open(env: &'static sudo_plugin::IoEnv) -> Result<Self> {
///         writeln!(env.stdout(), "example sudo plugin initialized");
///
///         Ok(Example { env })
///     }
///
///     fn close(self, _: i32, _: i32) {
///         writeln!(self.env.stdout(), "example sudo plugin exited");
///     }
///
///     fn log_stdout(&mut self, _: &[u8]) -> Result<()> {
///         writeln!(self.env.stdout(), "example sudo plugin received output on stdout");
///
///         Ok(())
///     }
/// }
/// # }
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
    ( $name:ident : $ty:ty ) => {
        mod $name {
            use super::*;

            // TODO: end use of static mut
            static mut SUDO_IO_ENV:    Option<$crate::IoEnv> = None;
            static mut SUDO_IO_PLUGIN: Option<$ty>           = None;

            pub struct State { }

            impl $crate::IoState<$ty> for State {
                unsafe fn io_env()    -> &'static mut Option<$crate::IoEnv> { &mut SUDO_IO_ENV }
                unsafe fn io_plugin() -> &'static mut Option<$ty>           { &mut SUDO_IO_PLUGIN }
            }
<<<<<<< HEAD
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
            let (mut stdout, mut stderr) = ::sudo_plugin::plugin::PrintFacility::new(
                Some(stringify!($name)), plugin_printf
            );
            let conversaton_f = ::sudo_plugin::plugin::ConversationFacility::new(
                conversation
            );

            let plugin = ::sudo_plugin::Plugin::new(
                stringify!($name).into(),
                option_env!("CARGO_PKG_VERSION").map(Into::into),
                version,
                argc, argv,
                settings_ptr,
                user_info_ptr,
                command_info_ptr,
                user_env_ptr,
                plugin_options_ptr,

                stdout,
                stderr.clone(), // we need stderr ourselves if `open` fails
                conversation,
                conversaton_f,
            );

            match plugin {
                Ok(p)  => PLUGIN = Some(p),
                Err(e) => {
                    let _ = stderr.write_error(&e);
                    return e.as_sudo_io_plugin_open_retval();
                },
            };

            // unwrap should be panic-safe here, since we just assigned
            // a value to $plugin
            let plugin = PLUGIN.as_ref().unwrap();

            // if the command is empty, to the best of my knowledge
            // we're being called with `-V` to report our version; in
            // this case there's no reason to fully invoke the plugin
            // through its `open` function
            if plugin.command_info.command == ::std::path::PathBuf::default() {
                return ::sudo_plugin::sys::SUDO_PLUGIN_OPEN_SUCCESS;
            }

            // call the plugin's `open` function
            match <$ty>::open(plugin) {
                Ok(i)  => INSTANCE = Some(i),
                Err(e) => {
                    let e: ::sudo_plugin::errors::Error = e.into();
                    let _ = stderr.write_error(&e);
                    return e.as_sudo_io_plugin_open_retval();
                },
            }

            ::sudo_plugin::sys::SUDO_PLUGIN_OPEN_SUCCESS
        }

        unsafe extern "C" fn close(
            _exit_status: ::libc::c_int,
            _error:       ::libc::c_int,
        ) {
            // force the instance to be dropped
            let _ = INSTANCE.take();
=======
>>>>>>> bd5e7daae1ab0536faf8c99a1bd1c183bfbdd3f0
        }

        #[allow(non_upper_case_globals)]
        #[allow(missing_docs)]
        #[no_mangle]
        pub static $name: $crate::sys::io_plugin = $crate::sys::io_plugin {
            type_:   $crate::sys::SUDO_IO_PLUGIN,
            version: $crate::sys::SUDO_API_VERSION,

            open:         Some($crate::core::open::<$ty, $name::State>),
            close:        Some($crate::core::close::<$ty, $name::State>),
            show_version: Some($crate::core::show_version::<$ty, $name::State>),

            log_ttyin:  Some($crate::core::log_ttyin ::<$ty, $name::State>),
            log_ttyout: Some($crate::core::log_ttyout::<$ty, $name::State>),
            log_stdin:  Some($crate::core::log_stdin ::<$ty, $name::State>),
            log_stdout: Some($crate::core::log_stdout::<$ty, $name::State>),
            log_stderr: Some($crate::core::log_stderr::<$ty, $name::State>),

            .. ::sudo_plugin::sys::IO_PLUGIN_EMPTY
        };
    }
}
