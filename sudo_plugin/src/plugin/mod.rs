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

//! Utilities for wrapping sudo plugins and the values they're
//! configured with.

mod option_map;
mod command_info;
mod settings;
mod user_info;
mod traits;

use super::errors::*;
use super::version::Version;

pub use self::option_map::OptionMap;

use self::command_info::CommandInfo;
use self::settings::Settings;
use self::user_info::UserInfo;

use sudo_plugin_sys;

use std::collections::HashSet;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::ffi::{CString, CStr};
use std::io::{self, Write};
use std::slice;
use std::sync::{Arc, Mutex};

use libc::{c_char, c_int, c_uint, gid_t};

/// An implementation of a sudo plugin, initialized and parsed from the
/// values passed to the underlying `open` callback.
#[allow(missing_debug_implementations)]
pub struct Plugin {
    /// The plugin API version supported by the invoked `sudo` command.
    pub version: Version,

    /// The command being executed, in the same form as would be passed
    /// to the `execve(2)` system call.
    pub command: Vec<CString>,

    /// A map of user-supplied sudo settings. These settings correspond
    /// to flags the user specified when running sudo. As such, they
    /// will only be present when the corresponding flag has been specified
    /// on the command line.
    pub settings: Settings,

    /// A map of information about the user running the command.
    pub user_info: UserInfo,

    /// A map of information about the command being run.
    pub command_info: CommandInfo,

    /// A map of the user's environment variables.
    pub user_env: OptionMap,

    /// A map of options provided to the plugin after the its path in
    /// sudo.conf.
    ///
    /// Settings that aren't of the form `key=value` will have a key
    /// in the map whose value is the same as the key, similar to how
    /// HTML handles valueless attributes (e.g., `disabled` will become
    /// `plugin_options["disabled"] => "disabled"`).
    pub plugin_options: OptionMap,

    printf:        Arc<Mutex<sudo_plugin_sys::sudo_printf_non_null_t>>,
    _conversation: sudo_plugin_sys::sudo_conv_t,
}

impl Plugin {
    /// Initializes a `Plugin` from the arguments provided to the
    /// underlying C `open` callback function. Verifies the API version
    /// advertised by the underlying `sudo` is supported by this library,
    /// parses all provided options, and wires up communication
    /// facilities.
    ///
    /// Returns an error if there was a problem initializing the plugin.
    #[cfg_attr(feature="cargo-clippy", allow(cast_sign_loss))]
    #[cfg_attr(feature="cargo-clippy", allow(too_many_arguments))]
    pub unsafe fn new(
        version:        c_uint,
        argc:           c_int,
        argv:           *const *const c_char,
        conversation:   sudo_plugin_sys::sudo_conv_t,
        plugin_printf:  sudo_plugin_sys::sudo_printf_t,
        settings:       *const *const c_char,
        user_info:      *const *const c_char,
        command_info:   *const *const c_char,
        user_env:       *const *const c_char,
        plugin_options: *const *const c_char,
    ) -> Result<Self> {
        let version = Version::from(version).check()?;

        // verify we've been given needed callbacks; we actually store the
        // Option-wrapped variants (instead of unwrapping them) because
        // those are the types the `sudo_plugin_sys` crate exports
        let printf = plugin_printf.ok_or(ErrorKind::Uninitialized)?;
        let _      = conversation .ok_or(ErrorKind::Uninitialized)?;

        // parse the argv into the command being run
        let mut argv    = slice::from_raw_parts(argv, argc as usize).to_vec();
        let     command = argv
            .drain(..)
            .map(|ptr| CStr::from_ptr(ptr).to_owned())
            .collect();

        let plugin = Self {
            version,
            command,

            // TODO(rust 1.27): convert `try_from` calls to `into` when
            // the TryFrom trait stabilizes
            settings:       Settings   ::try_from(OptionMap::from_raw(settings))?,
            user_info:      UserInfo   ::try_from(OptionMap::from_raw(user_info))?,
            command_info:   CommandInfo::try_from(OptionMap::from_raw(command_info))?,
            user_env:       OptionMap  ::from_raw(user_env),
            plugin_options: OptionMap  ::from_raw(plugin_options),

            printf: Arc::new(Mutex::new(printf)),
            _conversation: conversation,
        };

        Ok(plugin)
    }

    ///
    /// Returns a facility implementing `std::io::Write` that emits to
    /// the invoking user's STDOUT.
    ///
    pub fn stdout(&self) -> Printf {
        Printf {
            facility: self.printf.clone(),
            level:    sudo_plugin_sys::SUDO_CONV_INFO_MSG
        }
    }

    ///
    /// Returns a facility implementing `std::io::Write` that emits to
    /// the invoking user's STDERR.
    ///
    pub fn stderr(&self) -> Printf {
        Printf {
            facility: self.printf.clone(),
            level:    sudo_plugin_sys::SUDO_CONV_ERROR_MSG
        }
    }

    ///
    /// As best as can be reconstructed, what was actually typed at the
    /// shell in order to launch this invocation of sudo.
    ///
    // TODO: I don't really like this name
    pub fn invocation(&self) -> Vec<u8> {
        let mut sudo    = self.settings.progname.as_bytes().to_vec();
        let     flags   = self.settings.flags();
        let     command = self.command_info.command.as_os_str().as_bytes();

        if !flags.is_empty() {
            sudo.push(b' ');
            sudo.extend_from_slice(&flags.join(&b' ')[..]);
        }

        if !command.is_empty() {
            sudo.push(b' ');
            sudo.extend_from_slice(&command);
        }

        sudo
    }

    ///
    /// The `cwd` to be used for the command being run. This is
    /// typically set on the `user_info` component, but may be
    /// overridden by the policy plugin setting its value on
    /// `command_info`.
    ///
    pub fn cwd(&self) -> &PathBuf {
        self.command_info.cwd.as_ref().unwrap_or(
            &self.user_info.cwd
        )
    }

    ///
    /// The complete set of groups the invoked command will have
    /// privileges for. If the `-P` (`--preserve-groups`) flag was
    /// passed to `sudo`, the underlying `command_info` will not have
    /// this set and this method will return the list of original groups
    /// from the running the command.
    ///
    /// This set will always contain `runas_egid`.
    ///
    pub fn runas_gids(&self) -> HashSet<gid_t> {
        // sanity-check that if preserve_groups is unset we have
        // `runas_groups`, and if it is set that we don't
        if self.command_info.preserve_groups {
            debug_assert!(self.command_info.runas_groups.is_none())
        } else {
            debug_assert!(self.command_info.runas_groups.is_some())
        }

        // even though the above sanity-check might go wrong, it still
        // seems like a safe bet that if `runas_groups` isn't set that
        // the command will be invoked with the original user's groups
        // (it will probably require reading the `sudo` source code to
        // verify this)
        let mut set : HashSet<_> = self.command_info.runas_groups.as_ref().unwrap_or(
            &self.user_info.groups
        ).iter().cloned().collect();

        // `command_info.runas_egid` won't necessarily be in the list of
        // `command_info.runas_groups` if `-P` was passed; however, the
        // user will have this in the list of groups that they will gain
        // permissions for so it seems sane to include it in this list
        let _ = set.insert(self.command_info.runas_egid);

        set
    }
}

///
/// A facility implementing `std::io::Write` that allows printing
/// output to the user invoking `sudo`. Technically, the user may
/// not be present on a local tty, but this will be wired up to a
/// `printf`-like function that outputs to either STDOUT or STDERR.
///
#[derive(Debug)]
pub struct Printf {
    /// A *non-null* function pointer to a `sudo_printf_t` printf
    /// facility
    //
    // TODO: non-nullness should be validated here
    pub facility: Arc<Mutex<sudo_plugin_sys::sudo_printf_non_null_t>>,

    /// A `sudo_conv_message` bitflag to indicate how and where the
    /// message should be printed.
    //
    // TODO: level should be bitflags and validated
    pub level: u32,
}

impl Printf {
    ///
    /// Writes a formatted error to the user via the configured
    /// facility.
    ///
    pub fn write_error(&mut self, tag: &[u8], error: &Error) -> io::Result<()> {
        // errors are prefixed with a newline for clarity, since they
        // might be emitted while an existing line has output on it
        let mut message = vec![b'\n'];
        let mut stack   = vec![];

        // this is necessary since error_chain::Iter doesn't implement
        // `DoubleEndedIterator`, so we can't reverse it without pushing
        // everything onto a vec first
        for e in error.iter() {
            stack.push(e);
        }

        for e in stack.iter().rev() {
            message.extend_from_slice(tag);
            message.extend_from_slice(format!(": {}", e).as_bytes());
            message.push(b'\n');
        }

        self.write(&message[..]).and_then(|_| self.flush() )
    }
}

impl Write for Printf {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let message = CString::new(buf).map_err(|err|
            io::Error::new(io::ErrorKind::InvalidData, err)
        )?;

        let ret = unsafe {
            // TODO: this should be bitflags at some point, but the
            // cast is only necessary because Rust interprets the
            // `#define`'d constants as `u32` when they're treated by
            // sudo as `i32`.
            #[cfg_attr(feature="cargo-clippy", allow(cast_possible_wrap))]
            (self.facility.lock().unwrap())(self.level as i32, message.as_ptr())
        };

        if ret == -1 {
            Err(io::Error::last_os_error())?;
        }

        // TODO: replace the cast, but for now we've checked for it
        // being negative so there's no possibility of wraparound
        #[cfg_attr(feature="cargo-clippy", allow(cast_sign_loss))]
        Ok(ret as _)
    }

    // TODO: is there any meaningful implementation of this method?
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
