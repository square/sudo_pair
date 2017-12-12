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

use super::ffi::*;
use super::result::{Result, Error, ErrorKind};
use super::version::Version;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::io;
use std::str;

use libc::{c_char, c_uint};

#[macro_export]
macro_rules! sudo_io_plugin {
    ( $name:ident : $ty:ty { $($cb:ident : $fn:ident),* $(,)* } ) => {
        use sudo_plugin::result::AsSudoPluginRetval;

        static mut PLUGIN:   Option<sudo_plugin::Plugin> = None;
        static mut INSTANCE: Option<$ty>                 = None;

        #[no_mangle]
        #[allow(non_upper_case_globals)]
        #[allow(missing_docs)]
        pub static $name: sudo_plugin::ffi::io_plugin = {
            sudo_plugin::ffi::io_plugin {
                open: sudo_io_static_fn!(open, $name, PLUGIN, INSTANCE, $ty, open),

                $( $cb: sudo_io_fn!($cb, $name, PLUGIN, INSTANCE, $fn) ),*,

                .. sudo_plugin::ffi::io_plugin {
                    type_:            sudo_plugin::ffi::SUDO_IO_PLUGIN,
                    version:          sudo_plugin::ffi::SUDO_API_VERSION,
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

macro_rules! sudo_io_static_fn {
    ( open , $name:tt , $plugin:expr , $instance:expr , $ty:ty , $fn:ident ) => {{
        unsafe extern "C" fn sudo_plugin_open(
            version:            c_uint,
            conversation:       sudo_plugin::ffi::sudo_conv_t,
            plugin_printf:      sudo_plugin::ffi::sudo_printf_t,
            settings_ptr:       *const *mut c_char,
            user_info_ptr:      *const *mut c_char,
            command_info_ptr:   *const *mut c_char,
            _argc:              c_int,
            _argv:              *const *mut c_char,
            user_env_ptr:       *const *mut c_char,
            plugin_options_ptr: *const *mut c_char,
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
                .ok_or(sudo_plugin::result::Error::Simple(sudo_plugin::result::ErrorKind::Uninitialized))
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

pub struct Plugin {
    version: Version,

    pub settings:       HashMap<String, String>,
    pub user_info:      HashMap<String, String>,
    pub user_env:       HashMap<String, String>,
    pub command_info:   HashMap<String, String>,
    pub plugin_options: HashMap<String, String>,

    _conversation: sudo_conv_t,
    printf:        sudo_printf_t,
}

impl Plugin {
    #[cfg_attr(feature="clippy", allow(too_many_arguments))]
    pub fn new(
        version:        c_uint,
        conversation:   sudo_conv_t,
        plugin_printf:  sudo_printf_t,
        settings:       *const *mut c_char,
        user_info:      *const *mut c_char,
        command_info:   *const *mut c_char,
        user_env:       *const *mut c_char,
        plugin_options: *const *mut c_char,
    ) -> Plugin {
        let plugin = Plugin {
            version: Version::from(version),

            settings:       unsafe { parse_options(settings) },
            user_info:      unsafe { parse_options(user_info) },
            command_info:   unsafe { parse_options(command_info) },
            user_env:       unsafe { parse_options(user_env) },
            plugin_options: unsafe { parse_options(plugin_options) },

            _conversation: conversation,
            printf:        plugin_printf,
        };

        if plugin.version != Version::from(SUDO_API_VERSION) {
            let _ = plugin.print_error(format!(
                "sudo: WARNING: API version {}, built against version {}\n",
                version,
                SUDO_API_VERSION,
            ));
        }

        plugin
    }

    pub fn setting(&self, key: &str) -> Result<&str> {
        Self::fetch(&self.settings, "settings", key)
    }

    pub fn user_info(&self, key: &str) -> Result<&str> {
        Self::fetch(&self.user_info, "user_info", key)
    }

    // TODO: remove
    #[allow(dead_code)]
    pub fn user_env(&self, key: &str) -> Result<&str> {
        Self::fetch(&self.user_env, "user_env", key)
    }

    pub fn command_info(&self, key: &str) -> Result<&str> {
        Self::fetch(&self.command_info, "command_info", key)
    }

    // TODO: remove
    #[allow(dead_code)]
    pub fn plugin_options(&self, key: &str) -> Result<&str> {
        Self::fetch(&self.plugin_options, "plugin_options", key)
    }

    pub fn print_info<S: Borrow<str>>(&self, message: S) -> Result<()> {
        self.print(SUDO_CONV_FLAGS::INFO_MSG, message.borrow())
    }

    pub fn print_error<S: Borrow<str>>(&self, message: S) -> Result<()> {
        self.print(SUDO_CONV_FLAGS::ERROR_MSG, message.borrow())
    }

    fn print(&self, level: SUDO_CONV_FLAGS, message: &str) -> Result<()>{
        unsafe {
            let cstr = CString::new(message.as_bytes())?;
            let ret  = (self.printf)(level.bits(), cstr.as_ptr());

            if ret == -1 {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "failed to print to sudoer"
                ).into());
            }
        }

        Ok(())
    }

    fn fetch<'a>(map: &'a HashMap<String, String>, name: &str, key: &str) -> Result<&'a str> {
        map.get(key).ok_or_else(||
            Error::new(ErrorKind::MissingOption, format!("missing expected option {}[{}]", name, key))
        ).map(|v| v.as_str())
    }
}

unsafe fn parse_options(
    mut ptr: *const *mut c_char,
) -> HashMap<String, String> {
    let mut hash = HashMap::new();

    if ptr.is_null() {
        return hash;
    }

    while !(*ptr).is_null() {
        let bytes   = CStr::from_ptr(*ptr).to_bytes();
        let mid     = bytes.iter().position(|b| *b == b'=' ).unwrap_or_else(|| bytes.len());
        let (k, v)  = bytes.split_at(mid);

        // TODO: use [u8] instead of UTF-8 strings
        let key   = String::from_utf8(k     .to_vec()).expect("plugin key was not UTF-8");
        let value = String::from_utf8(v[1..].to_vec()).expect("plugin value was not UTF-8");

        let _ = hash.insert(key, value);

        ptr = ptr.offset(1);
    }

    hash
}
