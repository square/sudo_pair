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

use crate::errors::*;
use crate::options::OptionMap;
use crate::options::traits::{FromSudoOption, FromSudoOptionList};

use std::convert::TryFrom;
use std::net::{AddrParseError, IpAddr};
use std::str;

// TODO: copy all field-level documentation from `man sudo_plugin(8)`
#[derive(Debug)]
pub struct Settings {
    pub bsd_auth_type:        Option<String>,
    pub close_from:           Option<u64>,
    pub debug_flags:          Option<String>,
    pub debug_level:          Option<u64>,
    pub ignore_ticket:        bool,
    pub implied_shell:        bool,
    pub login_class:          Option<String>,
    pub login_shell:          bool,
    pub max_groups:           Option<u64>,
    pub network_addrs:        Vec<NetAddr>,
    pub noninteractive:       bool,
    pub plugin_dir:           String,
    pub plugin_path:          String,
    pub preserve_environment: bool,
    pub preserve_groups:      bool,
    pub progname:             String,
    pub prompt:               Option<String>,
    pub remote_host:          Option<String>,
    pub run_shell:            bool,
    pub runas_group:          Option<String>,
    pub runas_user:           Option<String>,
    pub selinux_role:         Option<String>,
    pub selinux_type:         Option<String>,
    pub set_home:             bool,
    pub sudoedit:             bool,
    pub timeout:              Option<String>,

    pub raw: OptionMap,
}

impl Settings {
    // TODO: surely this can be made more cleanly; also, it would be
    // great if we could actually get the full original `sudo`
    // invocation without having to reconstruct it by hand
    //
    // TODO: maybe if /proc/$$/cmd exists I can prefer to use it
    pub fn flags(&self) -> Vec<Vec<u8>> {
        let mut flags : Vec<Vec<u8>> = vec![];

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
