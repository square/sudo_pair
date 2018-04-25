use super::super::errors::*;
use super::option_map::*;
use super::traits::*;

use std::net::{AddrParseError, IpAddr};
use std::str;

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

    pub raw: OptionMap,
}

impl Settings {
    pub fn try_from(value: OptionMap) -> Result<Self> {
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

            raw: value,
        })
    }

    pub fn flags(&self) -> Vec<Vec<u8>> {
        let mut flags : Vec<Vec<u8>> = vec![];

        if let Some(ref runas_user) = self.runas_user {
            flags.push(b"-u".to_vec());
            flags.push(runas_user.as_bytes().to_vec());
        }

        if let Some(ref runas_group) = self.runas_group {
            flags.push(b"-g".to_vec());
            flags.push(runas_group.as_bytes().to_vec());
        }

        if self.login_shell {
            flags.push(b"-i".to_vec());
        }

        if self.run_shell {
            flags.push(b"-s".to_vec());
        }

        flags
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetAddr {
    pub addr: IpAddr,
    pub mask: IpAddr,
}

impl FromSudoOption for NetAddr {
    type Err = AddrParseError;

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
