mod option_map;
mod command_info;
mod settings;
mod user_info;

use super::errors::*;
use super::version::Version;

pub use self::option_map::OptionMap;

use self::command_info::CommandInfo;
use self::settings::Settings;
use self::user_info::UserInfo;

use sudo_plugin_sys;

use std::ffi::CString;

use libc::{c_char, c_int, c_uint};

#[allow(missing_debug_implementations)]
pub struct Plugin {
    pub version: Version,

    pub settings:       Settings,
    pub user_info:      UserInfo,
    pub command_info:   CommandInfo,
    pub user_env:       OptionMap,
    pub plugin_options: OptionMap,

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

        let settings       = Settings::new(OptionMap::new(settings)?)?;
        let user_info      = UserInfo::new(OptionMap::new(user_info)?)?;
        let command_info   = CommandInfo::new(OptionMap::new(command_info)?)?;
        let user_env       = OptionMap::new(user_env)?;
        let plugin_options = OptionMap::new(plugin_options)?;

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
            bail!(ErrorKind::IoError("plugin_printf".into()))
        }

        Ok(ret)
    }
}
