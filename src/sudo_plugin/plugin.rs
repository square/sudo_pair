// TODO: remove once actually using this
#![allow(dead_code)]

use super::ffi::*;

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::io::{Result, Error, ErrorKind};
use std::str;

use libc::{c_char, c_uint};

pub struct IoPlugin {
    version: Version,

    pub settings:       HashMap<String, String>,
    pub user_info:      HashMap<String, String>,
    pub user_env:       HashMap<String, String>,
    pub command_info:   HashMap<String, String>,
    pub plugin_options: HashMap<String, String>,

    conversation: sudo_conv_t,
    printf:       sudo_printf_t,
}

impl IoPlugin {
    pub fn new(
        version:        c_uint,
        conversation:   sudo_conv_t,
        plugin_printf:  sudo_printf_t,
        settings:       *const *mut c_char,
        user_info:      *const *mut c_char,
        command_info:   *const *mut c_char,
        user_env:       *const *mut c_char,
        plugin_options: *const *mut c_char,
    ) -> IoPlugin {
        let plugin = IoPlugin {
            version: Version::from(version),

            settings:       unsafe { parse_options(settings) },
            user_info:      unsafe { parse_options(user_info) },
            command_info:   unsafe { parse_options(command_info) },
            user_env:       unsafe { parse_options(user_env) },
            plugin_options: unsafe { parse_options(plugin_options) },

            conversation: conversation,
            printf:       plugin_printf,
        };

        if plugin.version != Version::from(SUDO_API_VERSION) {
            let _ = plugin.print_error(&format!(
                "sudo: WARNING: API version {}, built against version {}\n",
                version,
                SUDO_API_VERSION,
            ));
        }

        plugin
    }

    pub fn print_info(&self, message: &str) -> Result<()> {
        self.print(SUDO_CONV_INFO_MSG, message)
    }

    pub fn print_error(&self, message: &str) -> Result<()> {
        self.print(SUDO_CONV_ERROR_MSG, message)
    }

    fn print(&self, level: SUDO_CONV_FLAGS, message: &str) -> Result<()>{
        unsafe {
            let cstr = CString::new(message.as_bytes())?;
            let ret  = (self.printf)(level.bits(), cstr.as_ptr());

            if ret == -1 {
                return Err(Error::new(
                    ErrorKind::Other, "sudo_printf failed"
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Version {
    major: u16,
    minor: u16,
}

impl From<c_uint> for Version {
    fn from(version: c_uint) -> Self {
        Version{
            major: (version >> 16)     as u16,
            minor: (version &  0xffff) as u16,
        }
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
        let mid     = bytes.iter().position(|b| *b == b'=' ).unwrap_or(bytes.len());
        let (k, v)  = bytes.split_at(mid);

        // if the keys or values aren't UTF-8, panic; I considered
        // doing from_utf8_lossy here, but some values might
        // in theory be attacker-controlled, so better to die than
        // process something incorrectly
        let key   = String::from_utf8(k     .to_vec()).unwrap();
        let value = String::from_utf8(v[1..].to_vec()).unwrap();

        let _ = hash.insert(key, value);

        ptr = ptr.offset(1);
    }

    return hash;
}
