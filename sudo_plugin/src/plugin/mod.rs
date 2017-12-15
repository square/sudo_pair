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

use std::collections::HashMap;
use std::ffi::CString;
use std::io;

use libc::{c_char, c_int, c_uint};

pub struct Plugin {
    pub version: Version,

    pub settings:       Settings,
    pub user_info:      UserInfo,
    pub user_env:       HashMap<CString, CString>,
    pub command_info:   CommandInfo,
    pub plugin_options: HashMap<CString, CString>,

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
    ) -> Result<Self> {
        let version = Version::from(version).check()?;

        // verify we've been given needed callbacks; we store the
        // Option-wrapped variants (instead of unwrapping them) because
        // those are the types the `sudo_plugin_sys` crate exports
        let _ = plugin_printf.ok_or(ErrorKind::Uninitialized)?;
        let _ = conversation .ok_or(ErrorKind::Uninitialized)?;

        let settings       = Settings::new(settings)?;
        let user_info      = UserInfo::new(user_info)?;
        let command_info   = CommandInfo::new(command_info)?;
        let user_env       = parsing::parse_options(user_env)?;
        let plugin_options = parsing::parse_options(plugin_options)?;

        let plugin = Self {
            version,

            settings,
            user_info,
            command_info,
            user_env,
            plugin_options,

            _conversation: conversation,
            printf:        plugin_printf,
        };

        Ok(plugin)
    }

    pub fn print_info(&self, message: &str) -> Result<c_int> {
        self.print(sudo_plugin_sys::SUDO_CONV_INFO_MSG, message)
    }

    pub fn print_error(&self, message: &str) -> Result<c_int> {
        self.print(sudo_plugin_sys::SUDO_CONV_ERROR_MSG, message)
    }

    fn print(&self, level: c_uint, message: &str) -> Result<c_int> {
        unsafe {
            Self::printf(self.printf, level, message)
        }
    }

    // TODO: level should be bitflags
    pub unsafe fn printf<T: Into<Vec<u8>>>(
        printf:  sudo_plugin_sys::sudo_printf_t,
        level:   c_uint,
        message: T,
    ) -> Result<c_int> {
        let printf  = printf.ok_or(ErrorKind::Uninitialized)?;
        let cstring = CString::new(message.into())?;
        let ptr     = cstring.as_ptr();
        let ret     = (printf)(level as c_int, ptr);

        if ret == -1 {
            bail!(io::Error::new(io::ErrorKind::Other, "printing failed"))
        }

        Ok(ret)
    }
}
