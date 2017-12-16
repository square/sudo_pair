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
use std::ffi::{CString, CStr};
use std::net::{IpAddr, AddrParseError};
use std::os::unix::io::RawFd;
use std::result::Result as StdResult;
use std::str::FromStr;

use libc::{c_char, gid_t};

const OPTIONS_SEPARATOR : u8 = b'=';

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetAddr {
    pub addr: IpAddr,
    pub mask: IpAddr,
}

impl FromStr for NetAddr {
    type Err = AddrParseError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        let bytes = s.as_bytes();
        let mid   = bytes.iter()
            .position(|b| *b == b'/' )
            .unwrap_or(bytes.len());

        let addr = s[        .. mid].parse()?;
        let mask = s[mid + 1 ..    ].parse()?;

        Ok(Self { addr, mask })
    }
}

pub(crate) unsafe fn parse_options(
    mut ptr: *const *const c_char
) -> Result<HashMap<CString, CString>> {
    let mut map = HashMap::new();

    if ptr.is_null() {
         bail!("no settings were provided to the plugin")
    }

    while !(*ptr).is_null() {
        let mut bytes = CStr::from_ptr(*ptr).to_bytes_with_nul().to_owned();
        let sep       = bytes.iter().position(|b| *b == OPTIONS_SEPARATOR )
            .chain_err(|| "setting received by plugin has no separator" )?;

        // replace the separator with a NUL so we have two CStrings
        bytes[sep] = 0;

        let k = &bytes[        .. sep + 1];
        let v = &bytes[sep + 1 ..        ];

        // we don't need to check for NUL bytes in the key because we
        // put one there ourselves (in place of the separator), and
        // we don't need to check in the value because we're using the
        // one that was already there
        let key   = CStr::from_bytes_with_nul_unchecked(k).to_owned();
        let value = CStr::from_bytes_with_nul_unchecked(v).to_owned();

        let _ = map.insert(key, value);

        ptr = ptr.offset(1);
    }

    Ok(map)
}

pub(crate) fn parse_raw<T, F>(
    map:    &HashMap<CString, CString>,
    key:    &[u8],
    parser: F,
) -> Result<T>
    where F: FnOnce(&CStr) -> Option<T>
{
    let key = CStr::from_bytes_with_nul(key)
        .chain_err(|| "plugin author forgot a NUL" )?;

    map
        .get(key)
        .map(CString::as_c_str)
        .and_then(parser)
        .chain_err(|| format!("option {} wasn't provided to sudo_plugin", key.to_string_lossy()) )
}

pub(crate) fn parse<T: FromStr>(str: &CStr) -> Option<T> {
    str
        .to_str().ok()
        .and_then(|s| s.parse::<T>().ok() )
}

pub(crate) fn parse_fds(str: &CStr) -> Option<Vec<RawFd>> {
    parse_list(str, b',')
}

pub(crate) fn parse_gids(str: &CStr) -> Option<Vec<gid_t>> {
    parse_list(str, b',')
}

pub(crate) fn parse_net_addrs(str: &CStr) -> Option<Vec<NetAddr>> {
    parse_list(str, b' ')
}

fn parse_list<T: FromStr>(str: &CStr, sep: u8) -> Option<Vec<T>> {
    let list : Vec<&[u8]> = str.to_bytes()
        .split (|b| *b == sep )
        .collect();

    let mut items = Vec::with_capacity(list.len());

    for element in list {
        let str  = ::std::str::from_utf8(element).ok()?;
        let item = str.parse().ok()?;

        items.push(item);
    }

    Some(items)
}
