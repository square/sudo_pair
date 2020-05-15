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
