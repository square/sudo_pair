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

#![allow(missing_debug_implementations)]

mod parsing;
mod command_info;
mod settings;
mod user_info;

use super::errors::*;
use super::version::Version;

use self::command_info::CommandInfo;
use self::settings::Settings;
use self::user_info::UserInfo;

use sudo_plugin_sys;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::ffi::{CString, OsString};
use std::io;

use libc::{c_char, c_int, c_uint};

pub struct Plugin {
    version: Version,

    pub settings:       Settings,
    pub user_info:      UserInfo,
    pub user_env:       HashMap<OsString, OsString>,
    pub command_info:   CommandInfo,
    pub plugin_options: HashMap<OsString, OsString>,

    _conversation: sudo_plugin_sys::sudo_conv_t,
    printf:        sudo_plugin_sys::sudo_printf_t,
}

impl Plugin {
    #[cfg_attr(feature="clippy", allow(too_many_arguments))]
    pub unsafe fn new(
        version:        c_uint,
        conversation:   sudo_plugin_sys::sudo_conv_t,
        plugin_printf:  sudo_plugin_sys::sudo_printf_t,
        settings:       *const *const c_char,
        user_info:      *const *const c_char,
        command_info:   *const *const c_char,
        user_env:       *const *const c_char,
        plugin_options: *const *const c_char,
    ) -> Self {
        let plugin = Self {
            version: Version::from(version),

            // TODO: bail if !(version >= 1.2)

            // TODO: handle errors instead of dangerously unwrapping
            settings:       Settings::new(settings)       .unwrap(),
            user_info:      UserInfo::new(user_info)      .unwrap(),
            command_info:   CommandInfo::new(command_info).unwrap(),
            user_env:       parsing::parse_options(user_env).unwrap(),
            plugin_options: parsing::parse_options(plugin_options).unwrap(),

            _conversation: conversation,
            printf:        plugin_printf,
        };

        if plugin.version != Version::from(sudo_plugin_sys::SUDO_API_VERSION) {
            let _ = plugin.print_error(format!(
                "sudo: WARNING: API version {}, built against version {}\n",
                version,
                sudo_plugin_sys::SUDO_API_VERSION,
            ));
        }

        plugin
    }

    pub fn print_info<S: Borrow<str>>(&self, message: S) -> Result<()> {
        self.print(sudo_plugin_sys::SUDO_CONV_INFO_MSG, message.borrow())
    }

    pub fn print_error<S: Borrow<str>>(&self, message: S) -> Result<()> {
        self.print(sudo_plugin_sys::SUDO_CONV_ERROR_MSG, message.borrow())
    }

    fn print(&self, level: c_uint, message: &str) -> Result<()>{
        unsafe {
            let cstr   = CString::new(message)?;
            let printf = self.printf.ok_or(ErrorKind::MissingCallback("printf".to_string()))?;
            let ret    = (printf)(level as c_int, cstr.as_ptr());

            if ret == -1 {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "failed to print to sudoer"
                ).into());
            }
        }

        Ok(())
    }
}
