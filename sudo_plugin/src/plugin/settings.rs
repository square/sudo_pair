use super::super::errors::*;
use super::parsing;
use super::parsing::NetAddr;

use std::collections::HashMap;
use std::ffi::CString;

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

    pub raw: HashMap<CString, CString>,
}

impl Settings {
    pub fn new(ptr: *const *const c_char) -> Result<Self> {
        let raw = unsafe {
            parsing::parse_options(ptr)
        }?;

        Ok(Settings {
            plugin_dir:  parsing::parse_raw(&raw, b"plugin_dir\0",  parsing::parse)?,
            plugin_path: parsing::parse_raw(&raw, b"plugin_path\0", parsing::parse)?,
            progname:    parsing::parse_raw(&raw, b"progname\0",    parsing::parse)?,

            bsd_auth_type:        parsing::parse_raw(&raw, b"bsd_auth_type\0",        parsing::parse)          .ok(),
            close_from:           parsing::parse_raw(&raw, b"close_from\0",           parsing::parse)          .ok(),
            debug_flags:          parsing::parse_raw(&raw, b"debug_flags\0",          parsing::parse)          .ok(),
            debug_level:          parsing::parse_raw(&raw, b"debug_level\0",          parsing::parse)          .ok(),
            ignore_ticket:        parsing::parse_raw(&raw, b"ignore_ticket\0",        parsing::parse)          .unwrap_or(false),
            implied_shell:        parsing::parse_raw(&raw, b"implied_shell\0",        parsing::parse)          .unwrap_or(false),
            login_class:          parsing::parse_raw(&raw, b"login_class\0",          parsing::parse)          .ok(),
            login_shell:          parsing::parse_raw(&raw, b"login_shell\0",          parsing::parse)          .unwrap_or(false),
            max_groups:           parsing::parse_raw(&raw, b"max_groups\0",           parsing::parse)          .ok(),
            network_addrs:        parsing::parse_raw(&raw, b"network_addrs\0",        parsing::parse_net_addrs).unwrap_or(vec![]),
            noninteractive:       parsing::parse_raw(&raw, b"noninteractive\0",       parsing::parse)          .unwrap_or(false),
            preserve_environment: parsing::parse_raw(&raw, b"preserve_environment\0", parsing::parse)          .unwrap_or(false),
            preserve_groups:      parsing::parse_raw(&raw, b"preserve_groups\0",      parsing::parse)          .unwrap_or(false),
            prompt:               parsing::parse_raw(&raw, b"prompt\0",               parsing::parse)          .ok(),
            remote_host:          parsing::parse_raw(&raw, b"remote_host\0",          parsing::parse)          .ok(),
            run_shell:            parsing::parse_raw(&raw, b"run_shell\0",            parsing::parse)          .unwrap_or(false),
            runas_group:          parsing::parse_raw(&raw, b"runas_group\0",          parsing::parse)          .ok(),
            runas_user:           parsing::parse_raw(&raw, b"runas_user\0",           parsing::parse)          .ok(),
            selinux_role:         parsing::parse_raw(&raw, b"selinux_role\0",         parsing::parse)          .ok(),
            selinux_type:         parsing::parse_raw(&raw, b"selinux_type\0",         parsing::parse)          .ok(),
            set_home:             parsing::parse_raw(&raw, b"set_home\0",             parsing::parse)          .unwrap_or(false),
            sudoedit:             parsing::parse_raw(&raw, b"sudoedit\0",             parsing::parse)          .unwrap_or(false),

            raw: raw,
        })
    }
}
