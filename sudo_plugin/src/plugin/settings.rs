use super::super::errors::*;
use super::parsing;
use super::parsing::NetAddr;

use std::collections::HashMap;
use std::ffi::OsString;

use libc::c_char;

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

    pub raw: HashMap<OsString, OsString>,
}

impl Settings {
    pub fn new(ptr: *const *const c_char) -> Result<Self> {
        let raw = unsafe {
            parsing::parse_options(ptr)
        }?;

        Ok(Settings {
            plugin_dir:  parsing::parse_raw(&raw, "plugin_dir",  parsing::parse)?,
            plugin_path: parsing::parse_raw(&raw, "plugin_path", parsing::parse)?,
            progname:    parsing::parse_raw(&raw, "progname",    parsing::parse)?,

            bsd_auth_type:        parsing::parse_raw(&raw, "bsd_auth_type",        parsing::parse)          .ok(),
            close_from:           parsing::parse_raw(&raw, "close_from",           parsing::parse)          .ok(),
            debug_flags:          parsing::parse_raw(&raw, "debug_flags",          parsing::parse)          .ok(),
            debug_level:          parsing::parse_raw(&raw, "debug_level",          parsing::parse)          .ok(),
            ignore_ticket:        parsing::parse_raw(&raw, "ignore_ticket",        parsing::parse)          .unwrap_or(false),
            implied_shell:        parsing::parse_raw(&raw, "implied_shell",        parsing::parse)          .unwrap_or(false),
            login_class:          parsing::parse_raw(&raw, "login_class",          parsing::parse)          .ok(),
            login_shell:          parsing::parse_raw(&raw, "login_shell",          parsing::parse)          .unwrap_or(false),
            max_groups:           parsing::parse_raw(&raw, "max_groups",           parsing::parse)          .ok(),
            network_addrs:        parsing::parse_raw(&raw, "network_addrs",        parsing::parse_net_addrs).unwrap_or(vec![]),
            noninteractive:       parsing::parse_raw(&raw, "noninteractive",       parsing::parse)          .unwrap_or(false),
            preserve_environment: parsing::parse_raw(&raw, "preserve_environment", parsing::parse)          .unwrap_or(false),
            preserve_groups:      parsing::parse_raw(&raw, "preserve_groups",      parsing::parse)          .unwrap_or(false),
            prompt:               parsing::parse_raw(&raw, "prompt",               parsing::parse)          .ok(),
            remote_host:          parsing::parse_raw(&raw, "remote_host",          parsing::parse)          .ok(),
            run_shell:            parsing::parse_raw(&raw, "run_shell",            parsing::parse)          .unwrap_or(false),
            runas_group:          parsing::parse_raw(&raw, "runas_group",          parsing::parse)          .ok(),
            runas_user:           parsing::parse_raw(&raw, "runas_user",           parsing::parse)          .ok(),
            selinux_role:         parsing::parse_raw(&raw, "selinux_role",         parsing::parse)          .ok(),
            selinux_type:         parsing::parse_raw(&raw, "selinux_type",         parsing::parse)          .ok(),
            set_home:             parsing::parse_raw(&raw, "set_home",             parsing::parse)          .unwrap_or(false),
            sudoedit:             parsing::parse_raw(&raw, "sudoedit",             parsing::parse)          .unwrap_or(false),

            raw: raw,
        })
    }
}
