use super::super::errors::*;
use super::option_map::*;

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
    pub fn new(raw: OptionMap) -> Result<Self> {
        Ok(Self {
            plugin_dir:  raw.get_parsed("plugin_dir")?,
            plugin_path: raw.get_parsed("plugin_path")?,
            progname:    raw.get_parsed("progname")?,

            bsd_auth_type:        raw.get_parsed("bsd_auth_type")       .ok(),
            close_from:           raw.get_parsed("closefrom")           .ok(),
            debug_flags:          raw.get_parsed("debug_flags")         .ok(),
            debug_level:          raw.get_parsed("debug_level")         .ok(),
            ignore_ticket:        raw.get_parsed("ignore_ticket")       .unwrap_or(false),
            implied_shell:        raw.get_parsed("implied_shell")       .unwrap_or(false),
            login_class:          raw.get_parsed("login_class")         .ok(),
            login_shell:          raw.get_parsed("login_shell")         .unwrap_or(false),
            max_groups:           raw.get_parsed("max_groups")          .ok(),
            network_addrs:        raw.get_parsed("network_addrs")       .unwrap_or_else(|_| vec![]),
            noninteractive:       raw.get_parsed("noninteractive")      .unwrap_or(false),
            preserve_environment: raw.get_parsed("preserve_environment").unwrap_or(false),
            preserve_groups:      raw.get_parsed("preserve_groups")     .unwrap_or(false),
            prompt:               raw.get_parsed("prompt")              .ok(),
            remote_host:          raw.get_parsed("remote_host")         .ok(),
            run_shell:            raw.get_parsed("run_shell")           .unwrap_or(false),
            runas_group:          raw.get_parsed("runas_group")         .ok(),
            runas_user:           raw.get_parsed("runas_user")          .ok(),
            selinux_role:         raw.get_parsed("selinux_role")        .ok(),
            selinux_type:         raw.get_parsed("selinux_type")        .ok(),
            set_home:             raw.get_parsed("set_home")            .unwrap_or(false),
            sudoedit:             raw.get_parsed("sudoedit")            .unwrap_or(false),

            raw,
        })
    }

    // TODO: surely this can be made more cleanly
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
