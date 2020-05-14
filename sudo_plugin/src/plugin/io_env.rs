// Copyright 2020 Square Inc.
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

use crate::errors::*;
use crate::version::Version;
use crate::options::{OptionMap, CommandInfo, Settings, UserInfo};
use crate::output::{PrintFacility, Tty};

use std::convert::{TryFrom, TryInto};
use std::collections::HashSet;
use std::path::PathBuf;
use std::ffi::{CString, CStr};
use std::slice;

use libc::{c_char, c_int, c_uint, gid_t};

/// An implementation of the sudo io_plugin environment, initialized and
/// parsed from the values passed to the underlying `open` callback.
#[allow(missing_debug_implementations)]
pub struct IoEnv {
    /// The name of the plugin. This will be the generally be the same
    /// as the name of the exported C struct.
    pub plugin_name: &'static str,

    /// The version of the plugin.
    pub plugin_version: &'static str,

    /// The plugin API version supported by the invoked `sudo` command.
    pub api_version: Version,

    /// The command being executed under `sudo`, in the same form as
    /// would be passed to the `execve(2)` system call.
    pub cmdline: Vec<CString>,

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

    /// A handle to the plugin's printf_facility, configured to write to
    /// the user's stdout.
    stdout: PrintFacility,

    /// A handle to the plugin's printf_facility, configured to write to
    /// the user's stdin.
    stderr: PrintFacility,

    /// A (currently-unused) handle to the sudo_plugin conversation
    /// facility, which allows two-way communication with the user.
    _conversation: crate::sys::sudo_conv_t,
}

impl IoEnv {
    /// Initializes an `IoEnv` from the arguments provided to the
    /// underlying C `open` callback function.
    ///
    /// Verifies that the API version advertised by the underlying
    /// `sudo` is supported, parses all provided options, and wires up
    /// communication facilities.
    ///
    /// # Errors:
    ///
    /// Returns an error if there was a problem initializing the plugin.
    pub unsafe fn new(
        plugin_name:    &'static str,
        plugin_version: &'static str,
        version:        c_uint,
        argc:           c_int,
        argv:           *const *mut c_char,
        settings:       *const *mut c_char,
        user_info:      *const *mut c_char,
        command_info:   *const *mut c_char,
        user_env:       *const *mut c_char,
        plugin_options: *const *mut c_char,
        plugin_printf:  crate::sys::sudo_printf_t,
        conversation:   crate::sys::sudo_conv_t,
    ) -> Result<Self> {
        let version = Version::from(version).check()?;

        let (stdout, stderr) = PrintFacility::new(
            Some(plugin_name),
            plugin_printf
        );

        // parse the argv into the command being run
        let mut argv = slice::from_raw_parts(
            argv,
            argc as usize
        ).to_vec();

        let cmdline = argv
            .iter_mut()
            .map(|ptr| CStr::from_ptr(*ptr).to_owned())
            .collect();

        let plugin = Self {
            plugin_name,
            plugin_version,

            api_version: version,

            cmdline,

            settings:       OptionMap::from_raw(settings as _).try_into()?,
            user_info:      OptionMap::from_raw(user_info as _).try_into()?,
            command_info:   OptionMap::from_raw(command_info as _).try_into()?,
            user_env:       OptionMap::from_raw(user_env as _),
            plugin_options: OptionMap::from_raw(plugin_options as _),

            stdout,
            stderr,
            _conversation: conversation,
        };

        Ok(plugin)
    }

    ///
    /// Returns a facility implementing `std::io::Write` that emits to
    /// the invoking user's STDOUT.
    ///
    pub fn stdout(&self) -> PrintFacility {
        self.stdout.clone()
    }

    ///
    /// Returns a facility implementing `std::io::Write` that emits to
    /// the invoking user's STDERR.
    ///
    pub fn stderr(&self) -> PrintFacility {
        self.stderr.clone()
    }

    ///
    /// Returns a facility implementing `std::io::Write` that emits to
    /// the user's TTY, if sudo detected one.
    ///
    pub fn tty(&self) -> Option<Tty> {
        self.user_info.tty.as_ref().and_then(|path|
            Tty::try_from(path.as_path()).ok()
        )
    }

    ///
    /// As best as can be reconstructed, what was actually typed at the
    /// shell in order to launch this invocation of sudo.
    ///
    // TODO: I don't really like this name
    pub fn invocation(&self) -> Vec<u8> {
        let mut sudo    = self.settings.progname.as_bytes().to_vec();
        let     flags   = self.settings.flags();

        if !flags.is_empty() {
            sudo.push(b' ');
            sudo.extend_from_slice(&flags.join(&b' ')[..]);
        }

        for entry in &self.cmdline {
            sudo.push(b' ');
            sudo.extend_from_slice(entry.as_bytes());
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
