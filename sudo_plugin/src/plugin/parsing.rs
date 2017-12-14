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

pub(crate) fn parse_raw<T, F>(
    map:    &HashMap<Vec<u8>, CString>,
    key:    &str,
    parser: F,
) -> Result<T>
    where F: FnOnce(&CString) -> Option<T>
{
    map
        .get(key.as_bytes())
        .and_then(parser)
        .chain_err(|| format!("option {} wasn't provided to sudo_plugin", key) )
}

pub(crate) fn parse<T: FromStr>(str: &CString) -> Option<T> {
    str
        .to_str().ok()
        .and_then(|s| s.parse::<T>().ok() )
}

pub(crate) fn parse_fds(str: &CString) -> Option<Vec<RawFd>> {
    parse_list(str, b',')
}

pub(crate) fn parse_gids(str: &CString) -> Option<Vec<gid_t>> {
    parse_list(str, b',')
}

pub(crate) fn parse_net_addrs(str: &CString) -> Option<Vec<NetAddr>> {
    parse_list(str, b' ')
}

fn parse_list<T: FromStr>(str: &CString, sep: u8) -> Option<Vec<T>> {
    let list : Vec<CString> = str.to_bytes()
        .split (|b| *b == sep )
        .map   (|s| s.to_vec() )
        .map   (|v| unsafe { CString::from_vec_unchecked(v) } )
        .collect();

    let mut items = Vec::with_capacity(list.len());

    for element in list {
        items.push(element.to_str().ok()?.parse().ok()?);
    }

    Some(items)
}
