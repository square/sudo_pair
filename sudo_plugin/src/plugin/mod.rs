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
use std::ffi::{OsString, OsStr};
use std::io;
use std::os::unix::ffi::OsStrExt;

use libc::{c_char, c_int, c_uint};

#[allow(non_camel_case_types)]
type sudo_printf_t = unsafe extern "C" fn(c_int, *const c_char) -> c_int;

#[allow(non_camel_case_types)]
type sudo_conv_t   = unsafe extern "C" fn(c_int, *const sudo_plugin_sys::sudo_conv_message, *mut sudo_plugin_sys::sudo_conv_reply, *mut sudo_plugin_sys::sudo_conv_callback) -> c_int;

pub struct Plugin {
    pub version: Version,

    pub settings:       Settings,
    pub user_info:      UserInfo,
    pub user_env:       HashMap<OsString, OsString>,
    pub command_info:   CommandInfo,
    pub plugin_options: HashMap<OsString, OsString>,

    _conversation: sudo_conv_t,
    printf:        sudo_printf_t,
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

        let printf       = plugin_printf.ok_or(ErrorKind::MissingCallback("plugin_printf".into()))?;
        let conversation = conversation .ok_or(ErrorKind::MissingCallback("conversation".into()))?;

        let settings       = Settings::new(settings)               .chain_err(|| ErrorKind::Uninitialized )?;
        let user_info      = UserInfo::new(user_info)              .chain_err(|| ErrorKind::Uninitialized )?;
        let command_info   = CommandInfo::new(command_info)        .chain_err(|| ErrorKind::Uninitialized )?;
        let user_env       = parsing::parse_options(user_env)      .chain_err(|| ErrorKind::Uninitialized )?;
        let plugin_options = parsing::parse_options(plugin_options).chain_err(|| ErrorKind::Uninitialized )?;

        let plugin = Self {
            version,

            settings,
            user_info,
            command_info,
            user_env,
            plugin_options,

            _conversation: conversation,
            printf:        printf,
        };

        Ok(plugin)
    }

    pub fn print_info(&self, message: &str) -> Result<c_int> {
        self.print(sudo_plugin_sys::SUDO_CONV_INFO_MSG, message.borrow())
    }

    pub fn print_error(&self, message: &str) -> Result<c_int> {
        self.print(sudo_plugin_sys::SUDO_CONV_ERROR_MSG, message.borrow())
    }

    fn print<S: AsRef<OsStr>>(&self, level: c_uint, message: S) -> Result<c_int> {
        unsafe {
            Self::printf(self.printf, level, message.as_ref())
        }
    }

    // TODO: level should be bitflags
    pub unsafe fn printf(
        printf:  sudo_printf_t,
        level:   c_uint,
        message: &OsStr,
    ) -> Result<c_int> {
        let ptr = message.as_bytes().as_ptr();
        let ret = (printf)(level as c_int, ptr as *const _);

        // TODO: bail!
        if ret == -1 {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "printing failed"
            ).into());
        }

        Ok(ret)
    }
}
