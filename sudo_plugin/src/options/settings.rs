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

use crate::errors::{Result, Error};
use crate::options::OptionMap;
use crate::options::traits::{FromSudoOption, FromSudoOptionList};

use std::convert::TryFrom;
use std::net::{AddrParseError, IpAddr};
use std::str;

/// A vector of user-supplied sudo settings. These settings correspond
/// to options the user specified when running sudo. As such, they will
/// only be present when the corresponding option has been specified on
/// the command line.
#[derive(Debug)]
pub struct Settings {
    /// Authentication type, if specified by the -a option, to use on systems
    /// where BSD authentication is supported.
    pub bsd_auth_type: Option<String>,

    /// If specified, the user has requested via the -C option that sudo close
    /// all files descriptors with a value of number or higher. The plugin may
    /// optionally pass this, or another value, back in the command_info list.
    pub close_from: Option<u64>,

    /// A debug file path name followed by a space and a comma-separated list
    /// of debug flags that correspond to the plugin's Debug entry in
    /// sudo.conf(5), if there is one. The flags are passed to the plugin
    /// exactly as they appear in sudo.conf(5). The syntax used by sudo and the
    /// sudoers plugin is subsystem@priority but a plugin is free to use a
    /// different format so long as it does not include a comma (‘,’). Prior to
    /// sudo 1.8.12, there was no way to specify plugin-specific debug_flags so
    /// the value was always the same as that used by the sudo front end and
    /// did not include a path name, only the flags themselves. As of version
    /// 1.7 of the plugin interface, sudo will only pass debug_flags if
    /// sudo.conf(5) contains a plugin-specific Debug entry.
    pub debug_flags: Option<String>,

    /// This setting has been deprecated in favor of debug_flags.
    pub debug_level: Option<u64>,

    /// Set to true if the user specified the -k option along with a command,
    /// indicating that the user wishes to ignore any cached authentication
    /// credentials. implied_shell to true. This allows sudo with no arguments
    /// to be used similarly to su(1). If the plugin does not to support this
    /// usage, it may return a value of -2 from the check_policy() function,
    /// which will cause sudo to print a usage message and exit.
    pub ignore_ticket: bool,

    /// If the user does not specify a program on the command line, sudo will
    /// pass the plugin the path to the user's shell and set
    pub implied_shell: bool,

    /// BSD login class to use when setting resource limits and nice value, if
    /// specified by the -c option.
    pub login_class: Option<String>,

    /// Set to true if the user specified the -i option, indicating that the
    /// user wishes to run a login shell.
    pub login_shell: bool,

    /// The maximum number of groups a user may belong to. This will only be
    /// present if there is a corresponding setting in sudo.conf(5).
    pub max_groups: Option<u64>,

    /// A space-separated list of IP network addresses and netmasks in the form
    /// “addr/netmask”, e.g., “192.168.1.2/255.255.255.0”. The address and
    /// netmask pairs may be either IPv4 or IPv6, depending on what the
    /// operating system supports. If the address contains a colon (‘:’), it is
    /// an IPv6 address, else it is IPv4.
    pub network_addrs: Vec<NetAddr>,

    /// Set to true if the user specified the -n option, indicating that sudo
    /// should operate in non-interactive mode. The plugin may reject a command
    /// run in non-interactive mode if user interaction is required.
    pub noninteractive: bool,

    /// The default plugin directory used by the sudo front end. This is the
    /// default directory set at compile time and may not correspond to the
    /// directory the running plugin was loaded from. It may be used by a
    /// plugin to locate support files.
    pub plugin_dir: String,

    /// The path name of plugin loaded by the sudo front end. The path name
    /// will be a fully-qualified unless the plugin was statically compiled
    /// into sudo.
    pub plugin_path: String,

    /// Set to true if the user specified the -E option, indicating that the
    /// user wishes to preserve the environment.
    pub preserve_environment: bool,

    /// Set to true if the user specified the -P option, indicating that the
    /// user wishes to preserve the group vector instead of setting it based on
    /// the runas user.
    pub preserve_groups: bool,

    /// The command name that sudo was run as, typically “sudo” or “sudoedit”.
    pub progname: String,

    /// The prompt to use when requesting a password, if specified via the -p
    /// option.
    pub prompt: Option<String>,

    /// The name of the remote host to run the command on, if specified via the
    /// -h option. Support for running the command on a remote host is meant to
    /// be implemented via a helper program that is executed in place of the
    /// user-specified command. The sudo front end is only capable of executing
    /// commands on the local host. Only available starting with API version
    /// 1.4.
    pub remote_host: Option<String>,

    /// Set to true if the user specified the -s option, indicating that the
    /// user wishes to run a shell.
    pub run_shell: bool,

    /// The group name or gid to run the command as, if specified via the -g
    /// option.
    pub runas_group: Option<String>,

    /// The user name or uid to run the command as, if specified via the -u
    /// option.
    pub runas_user: Option<String>,

    /// SELinux role to use when executing the command, if specified by the -r
    /// option.
    pub selinux_role: Option<String>,

    /// SELinux type to use when executing the command, if specified by the -t
    /// option.
    pub selinux_type: Option<String>,

    /// Set to true if the user specified the -H option. If true, set the HOME
    /// environment variable to the target user's home directory.
    pub set_home: bool,

    /// Set to true when the -e option is specified or if invoked as sudoedit.
    /// The plugin shall substitute an editor into argv in the check_policy()
    /// function or return -2 with a usage error if the plugin does not support
    /// sudoedit. For more information, see the check_policy section.
    pub sudoedit: bool,

    /// User-specified command timeout. Not all plugins support command
    /// timeouts and the ability for the user to set a timeout may be
    /// restricted by policy. The format of the timeout string is
    /// plugin-specific.
    pub timeout: Option<String>,

    /// The raw underlying [`OptionMap`](OptionMap) to retrieve additional
    /// values that may not have been known at the time of the authorship of
    /// this file.
    pub raw: OptionMap,
}

impl Settings {
    // TODO: surely this can be made more cleanly; also, it would be
    // great if we could actually get the full original `sudo`
    // invocation without having to reconstruct it by hand
    //
    // TODO: maybe if /proc/$$/cmd exists I can prefer to use it
    #[must_use]
    pub fn flags(&self) -> Vec<Vec<u8>> {
        let mut flags: Vec<Vec<u8>> = vec![];

        // `sudoedit` is set if the flag was provided *or* if sudo
        // was invoked as `sudoedit` directly; try our best to intrepret
        // this case, although we'll technically get it wrong in the
        // case of `sudoedit -e ...`
        if self.sudoedit && self.progname != "sudoedit" {
            flags.push(b"--edit".to_vec());
        }

        if let Some(ref runas_user) = self.runas_user {
            let mut flag = b"--user ".to_vec();
            flag.extend_from_slice(runas_user.as_bytes());

            flags.push(flag);
        }

        if let Some(ref runas_group) = self.runas_group {
            let mut flag = b"--group ".to_vec();
            flag.extend_from_slice(runas_group.as_bytes());

            flags.push(flag);
        }

        if let Some(ref prompt) = self.prompt {
            let mut flag = b"--prompt ".to_vec();
            flag.extend_from_slice(prompt.as_bytes());

            flags.push(flag);
        }

        if self.login_shell {
            flags.push(b"--login".to_vec());
        }

        if self.run_shell {
            flags.push(b"--shell".to_vec());
        }

        if self.set_home {
            flags.push(b"--set-home".to_vec());
        }

        if self.preserve_environment {
            flags.push(b"--preserve-env".to_vec());
        }

        if self.preserve_groups {
            flags.push(b"--preserve-groups".to_vec());
        }

        if self.ignore_ticket {
            flags.push(b"--reset-timestamp".to_vec());
        }

        if self.noninteractive {
            flags.push(b"--non-interactive".to_vec());
        }

        if let Some(ref login_class) = self.login_class {
            let mut flag = b"--login-class ".to_vec();
            flag.extend_from_slice(login_class.as_bytes());

            flags.push(flag);
        }

        if let Some(ref selinux_role) = self.selinux_role {
            let mut flag = b"--role ".to_vec();
            flag.extend_from_slice(selinux_role.as_bytes());

            flags.push(flag);
        }

        if let Some(ref selinux_type) = self.selinux_type {
            let mut flag = b"--type ".to_vec();
            flag.extend_from_slice(selinux_type.as_bytes());

            flags.push(flag);
        }

        if let Some(ref bsd_auth_type) = self.bsd_auth_type {
            let mut flag = b"--auth-type ".to_vec();
            flag.extend_from_slice(bsd_auth_type.as_bytes());

            flags.push(flag);
        }

        if let Some(close_from) = self.close_from {
            let mut flag = b"--close-from ".to_vec();
            flag.extend_from_slice(close_from.to_string().as_bytes());

            flags.push(flag);
        }

        flags
    }
}

impl TryFrom<OptionMap> for Settings {
    type Error = Error;

    fn try_from(value: OptionMap) -> Result<Self> {
        Ok(Self {
            plugin_dir:  value.get("plugin_dir")?,
            plugin_path: value.get("plugin_path")?,
            progname:    value.get("progname")?,

            bsd_auth_type:        value.get("bsd_auth_type")       .ok(),
            close_from:           value.get("closefrom")           .ok(),
            debug_flags:          value.get("debug_flags")         .ok(),
            debug_level:          value.get("debug_level")         .ok(),
            ignore_ticket:        value.get("ignore_ticket")       .unwrap_or(false),
            implied_shell:        value.get("implied_shell")       .unwrap_or(false),
            login_class:          value.get("login_class")         .ok(),
            login_shell:          value.get("login_shell")         .unwrap_or(false),
            max_groups:           value.get("max_groups")          .ok(),
            network_addrs:        value.get("network_addrs")       .unwrap_or_else(|_| vec![]),
            noninteractive:       value.get("noninteractive")      .unwrap_or(false),
            preserve_environment: value.get("preserve_environment").unwrap_or(false),
            preserve_groups:      value.get("preserve_groups")     .unwrap_or(false),
            prompt:               value.get("prompt")              .ok(),
            remote_host:          value.get("remote_host")         .ok(),
            run_shell:            value.get("run_shell")           .unwrap_or(false),
            runas_group:          value.get("runas_group")         .ok(),
            runas_user:           value.get("runas_user")          .ok(),
            selinux_role:         value.get("selinux_role")        .ok(),
            selinux_type:         value.get("selinux_type")        .ok(),
            set_home:             value.get("set_home")            .unwrap_or(false),
            sudoedit:             value.get("sudoedit")            .unwrap_or(false),
            timeout:              value.get("timeout")             .ok(),

            raw: value,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetAddr {
    pub addr: IpAddr,
    pub mask: IpAddr,
}

impl FromSudoOption for NetAddr {
    type Err = AddrParseError;

    // indexing into an array can panic, but there's no cleaner way in
    // rust to split a byte array on a delimiter, and the code below
    // selects the midpoint such that it's guaranteed to be within the
    // slice
    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        let mid   = bytes.iter()
            .position(|b| *b == b'/' )
            .unwrap_or_else(|| bytes.len());

        let addr = s[        .. mid].parse()?;
        let mask = s[mid + 1 ..    ].parse()?;

        Ok(Self {
            addr,
            mask,
        })
    }
}

impl FromSudoOptionList for NetAddr {
    const SEPARATOR: char = ' ';
}
