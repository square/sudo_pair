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

use super::super::errors::*;

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::net::IpAddr;

use libc::c_char;

const OPTIONS_SEPARATOR          : u8 = b'=';
const OPTIONS_SEPARATOR_LIST     : u8 = b' ';
const OPTIONS_SEPARATOR_NET_ADDR : u8 = b'/';

#[derive(Debug)]
pub struct Settings {
    pub bsd_auth_type :        Option<CString>,
    pub close_from :           Option<u64>,
    pub debug_flags :          Option<CString>,
    pub debug_level :          Option<u64>,
    pub ignore_ticket :        bool,
    pub implied_shell :        bool,
    pub login_class :          Option<CString>,
    pub login_shell :          bool,
    pub max_groups :           Option<u64>,
    pub network_addrs :        Vec<NetAddr>,
    pub noninteractive :       bool,
    pub plugin_dir :           CString,
    pub plugin_path :          CString,
    pub preserve_environment : bool,
    pub preserve_groups :      bool,
    pub progname :             CString,
    pub prompt :               Option<CString>,
    pub remote_host :          Option<CString>,
    pub run_shell :            bool,
    pub runas_group :          Option<CString>,
    pub runas_user :           CString,
    pub selinux_role :         Option<CString>,
    pub selinux_type :         Option<CString>,
    pub set_home :             bool,
    pub sudoedit :             bool,

    pub other: HashMap<Vec<u8>, CString>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bsd_auth_type:        None,
            close_from:           None,
            debug_flags:          None,
            debug_level:          None,
            ignore_ticket:        false,
            implied_shell:        false,
            login_class:          None,
            login_shell:          false,
            max_groups:           None,
            network_addrs:        vec![],
            noninteractive:       false,
            plugin_dir:           unsafe { CString::from_vec_unchecked(vec![0]) },
            plugin_path:          unsafe { CString::from_vec_unchecked(vec![0]) },
            preserve_environment: false,
            preserve_groups:      false,
            progname:             unsafe { CString::from_vec_unchecked(vec![0]) },
            prompt:               None,
            remote_host:          None,
            run_shell:            false,
            runas_group:          None,
            runas_user:           unsafe { CString::from_vec_unchecked(b"root".to_vec()) },
            selinux_role:         None,
            selinux_type:         None,
            set_home:             false,
            sudoedit:             false,

            other: HashMap::new(),
        }
    }
}

impl Settings {
    pub fn new(ptr: *const *const c_char) -> Result<Self> {
        let mut settings = Self::default();

        unsafe {
            parse_options(ptr, |key, value| {
                match key {
                    b"bsdauth_type"         => settings.bsd_auth_type        = parse_string(value),
                    b"closefrom"            => settings.close_from           = parse_u64(value),
                    b"debug_flags"          => settings.debug_flags          = parse_string(value),
                    b"debug_level"          => settings.debug_level          = parse_u64(value),
                    b"ignore_ticket"        => settings.ignore_ticket        = parse_bool(value),
                    b"implied_shell"        => settings.implied_shell        = parse_bool(value),
                    b"login_class"          => settings.login_class          = parse_string(value),
                    b"login_shell"          => settings.login_shell          = parse_bool(value),
                    b"max_groups"           => settings.max_groups           = parse_u64(value),
                    b"network_addrs"        => settings.network_addrs        = parse_net_list(value),
                    b"noninteractive"       => settings.noninteractive       = parse_bool(value),
                    b"plugin_dir"           => settings.plugin_dir           = value.to_owned(),
                    b"plugin_path"          => settings.plugin_path          = value.to_owned(),
                    b"preserve_environment" => settings.preserve_environment = parse_bool(value),
                    b"preserve_groups"      => settings.preserve_groups      = parse_bool(value),
                    b"progname"             => settings.progname             = value.to_owned(),
                    b"prompt"               => settings.prompt               = parse_string(value),
                    b"remote_host"          => settings.remote_host          = parse_string(value),
                    b"run_shell"            => settings.run_shell            = parse_bool(value),
                    b"runas_group"          => settings.runas_group          = parse_string(value),
                    b"runas_user"           => settings.runas_user           = value.to_owned(),
                    b"selinux_role"         => settings.selinux_role         = parse_string(value),
                    b"selinux_type"         => settings.selinux_type         = parse_string(value),
                    b"set_home"             => settings.set_home             = parse_bool(value),
                    b"sudoedit"             => settings.sudoedit             = parse_bool(value),
                    _                       => { let _ = settings.other.insert(key.to_vec(), value.to_owned()); },
                };
            })?;
        }

        Ok(settings)
    }
}

unsafe fn parse_options<F>(
    mut ptr: *const *const c_char,
    mut cb:  F
) -> Result<()> where F: FnMut(&[u8], &CStr) {
    if ptr.is_null() {
        bail!("no settings were provided to the plugin")
    }

    while !(*ptr).is_null() {
        let bytes = CStr::from_ptr(*ptr).to_bytes_with_nul();
        let mid   = bytes.iter().position(|b| *b == OPTIONS_SEPARATOR )
            .chain_err(|| "setting received by plugin has no separator" )?;

        let k = &bytes[        .. mid];
        let v = &bytes[mid + 1 ..    ];

        let value = CStr::from_bytes_with_nul(v)
            .chain_err(|| "setting received by plugin was malformed" )?;

        cb(k, &value);

        ptr = ptr.offset(1);
    }

    Ok(())
}

fn parse_string(str: &CStr) -> Option<CString> {
    Some(str.to_owned())
}

fn parse_bool(str: &CStr) -> bool {
    str.to_str().ok()
        .and_then(|s| s.parse::<bool>().ok() )
        .unwrap_or(false)
}

fn parse_u64(str: &CStr) -> Option<u64> {
    str.to_str().ok()
        .and_then(|s| s.parse::<u64>().ok() )
}

fn parse_list(str: &CStr) -> Vec<CString> {
    str.to_bytes()
        .split (|b| *b == OPTIONS_SEPARATOR_LIST )
        .map   (|s| s.to_vec() )
        .map   (|v| unsafe { CString::from_vec_unchecked(v) } )
        .collect()
}

fn parse_net_list(str: &CStr) -> Vec<NetAddr> {
    parse_list(str).iter().map (|entry| {
        let bytes = entry.to_bytes();
        let mid   = bytes.iter()
            .position(|b| *b == OPTIONS_SEPARATOR_NET_ADDR )
            .unwrap_or(bytes.len());

        let addr = entry.to_string_lossy()[        .. mid].parse();
        let mask = entry.to_string_lossy()[mid + 1 ..    ].parse();

        if addr.is_err() || mask.is_err() {
            return None;
        }

        Some(NetAddr {
            addr: addr.unwrap(),
            mask: mask.unwrap(),
        })
    }).filter(|o| o.is_some() ).map(|o| o.unwrap() ).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetAddr {
    pub addr: IpAddr,
    pub mask: IpAddr,
}
