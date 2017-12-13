#![allow(missing_debug_implementations)]

mod settings;

use super::errors::*;
use super::version::Version;
use self::settings::Settings;

use sudo_plugin_sys;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::io;

use libc::{c_char, c_int, c_uint};

pub struct Plugin {
    version: Version,

    pub settings:       Settings,
    pub user_info:      HashMap<String, String>,
    pub user_env:       HashMap<String, String>,
    pub command_info:   HashMap<String, String>,
    pub plugin_options: HashMap<String, String>,

    _conversation: sudo_plugin_sys::sudo_conv_t,
    printf:        sudo_plugin_sys::sudo_printf_t,
}

impl Plugin {
    #[cfg_attr(feature="clippy", allow(too_many_arguments))]
    pub fn new(
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

            // TODO: handle errors instead of dangerously unwrapping
            settings:       Settings::new(settings).unwrap(),
            user_info:      unsafe { parse_options_old(user_info) },
            command_info:   unsafe { parse_options_old(command_info) },
            user_env:       unsafe { parse_options_old(user_env) },
            plugin_options: unsafe { parse_options_old(plugin_options) },

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

    fn fetch<'a>(map: &'a HashMap<String, String>, name: &str, key: &str) -> Result<&'a str> {
        map.get(key).ok_or_else(||
            ErrorKind::MissingOption(name.to_string(), key.to_string()).into()
        ).map(|v| v.as_str())
    }
}

unsafe fn parse_options_old(
    mut ptr: *const *const c_char,
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
