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
    pub bsd_auth_type :        Option<String>,
    pub close_from :           Option<u64>,
    pub debug_flags :          Option<String>,
    pub debug_level :          Option<u64>,
    pub ignore_ticket :        bool,
    pub implied_shell :        bool,
    pub login_class :          Option<String>,
    pub login_shell :          bool,
    pub max_groups :           Option<u64>,
    pub network_addrs :        Vec<NetAddr>,
    pub noninteractive :       bool,
    pub plugin_dir :           String,
    pub plugin_path :          String,
    pub preserve_environment : bool,
    pub preserve_groups :      bool,
    pub progname :             String,
    pub prompt :               Option<String>,
    pub remote_host :          Option<String>,
    pub run_shell :            bool,
    pub runas_group :          Option<String>,
    pub runas_user :           Option<String>,
    pub selinux_role :         Option<String>,
    pub selinux_type :         Option<String>,
    pub set_home :             bool,
    pub sudoedit :             bool,

    pub raw : HashMap<Vec<u8>, CString>,
}

impl Settings {
    pub fn new(ptr: *const *const c_char) -> Result<Self> {
        let raw = unsafe {
            parse_options(ptr)
        }?;

        Ok(Settings {
            plugin_dir:  raw.get(&b"plugin_dir"[..]) .and_then(parse_string).chain_err(|| "missing plugin_dir") ?,
            plugin_path: raw.get(&b"plugin_path"[..]).and_then(parse_string).chain_err(|| "missing plugin_path")?,
            progname:    raw.get(&b"progname"[..])   .and_then(parse_string).chain_err(|| "missing progname")   ?,

            bsd_auth_type:        raw.get(&b"bsduth_type"[..])         .and_then(parse_string),
            close_from:           raw.get(&b"closefrom"[..])           .and_then(parse_u64),
            debug_flags:          raw.get(&b"debug_flags"[..])         .and_then(parse_string),
            debug_level:          raw.get(&b"debug_level"[..])         .and_then(parse_u64),
            ignore_ticket:        raw.get(&b"ignore_ticket"[..])       .and_then(parse_bool)     .unwrap_or(false),
            implied_shell:        raw.get(&b"implied_shell"[..])       .and_then(parse_bool)     .unwrap_or(false),
            login_class:          raw.get(&b"login_class"[..])         .and_then(parse_string),
            login_shell:          raw.get(&b"login_shell"[..])         .and_then(parse_bool)     .unwrap_or(false),
            max_groups:           raw.get(&b"max_groups"[..])          .and_then(parse_u64),
            network_addrs:        raw.get(&b"network_addrs"[..])       .and_then(parse_net_addrs).unwrap_or(vec![]),
            noninteractive:       raw.get(&b"noninteractive"[..])      .and_then(parse_bool)     .unwrap_or(false),
            preserve_environment: raw.get(&b"preserve_environment"[..]).and_then(parse_bool)     .unwrap_or(false),
            preserve_groups:      raw.get(&b"preserve_groups"[..])     .and_then(parse_bool)     .unwrap_or(false),
            prompt:               raw.get(&b"prompt"[..])              .and_then(parse_string),
            remote_host:          raw.get(&b"remote_host"[..])         .and_then(parse_string),
            run_shell:            raw.get(&b"run_shell"[..])           .and_then(parse_bool)     .unwrap_or(false),
            runas_group:          raw.get(&b"runas_group"[..])         .and_then(parse_string),
            runas_user:           raw.get(&b"runas_user"[..])          .and_then(parse_string),
            selinux_role:         raw.get(&b"selinux_role"[..])        .and_then(parse_string),
            selinux_type:         raw.get(&b"selinux_type"[..])        .and_then(parse_string),
            set_home:             raw.get(&b"set_home"[..])            .and_then(parse_bool)     .unwrap_or(false),
            sudoedit:             raw.get(&b"sudoedit"[..])            .and_then(parse_bool)     .unwrap_or(false),

            raw: raw,
        })
    }
}

unsafe fn parse_options(
    mut ptr: *const *const c_char
) -> Result<HashMap<Vec<u8>, CString>> {
    let mut map = HashMap::new();

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

        let _ = map.insert(
            k    .to_owned(),
            value.to_owned()
        );

        ptr = ptr.offset(1);
    }

    Ok(map)
}

fn parse_string(str: &CString) -> Option<String> {
    str.to_owned().into_string().ok()
}

fn parse_bool(str: &CString) -> Option<bool> {
    str
        .to_str().ok()
        .and_then(|s| s.parse::<bool>().ok() )}

fn parse_u64(str: &CString) -> Option<u64> {
    str
        .to_str().ok()
        .and_then(|s| s.parse::<u64>().ok() )
}

fn parse_net_addrs(str: &CString) -> Option<Vec<NetAddr>> {
    let net_addrs : Vec<Option<NetAddr>> = parse_list(str).iter().map (|entry| {
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
    }).collect();

    Some(
        net_addrs.iter()
            .filter(|o| o.is_some() )
            .map   (|o| o.unwrap() )
            .collect()
    )
}

fn parse_list(str: &CString) -> Vec<CString> {
    str.to_bytes()
        .split (|b| *b == OPTIONS_SEPARATOR_LIST )
        .map   (|s| s.to_vec() )
        .map   (|v| unsafe { CString::from_vec_unchecked(v) } )
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetAddr {
    pub addr: IpAddr,
    pub mask: IpAddr,
}
