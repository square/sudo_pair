use super::super::errors::*;

use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::str::{self, FromStr};

use libc::c_char;

const OPTIONS_SEPARATOR : u8 = b'=';

#[derive(Clone, Debug)]
pub struct RawOptions(HashMap<Vec<u8>, Vec<u8>>);

#[derive(Clone, Copy)]
pub struct ParseListError();

impl RawOptions {
    pub unsafe fn new(mut ptr: *const *const c_char) -> Result<Self> {
        let mut map = HashMap::new();

        if ptr.is_null() {
            bail!("no settings were provided to the plugin")
        }

        while !(*ptr).is_null() {
            let bytes = CStr::from_ptr(*ptr).to_bytes();
            let sep   = bytes.iter().position(|b| *b == OPTIONS_SEPARATOR )
                .chain_err(|| "setting received by plugin has no separator" )?;

            let key   = bytes[        .. sep].to_owned();
            let value = bytes[sep + 1 ..    ].to_owned();

            let _ = map.insert(key, value);

            ptr = ptr.offset(1);
        }

        Ok(RawOptions(map))
    }

    pub fn get(&self, k: &str) -> Option<&str> {
        self.get_raw(k.as_bytes())
            .and_then(|b| str::from_utf8(b).ok() )
    }

    pub fn get_parsed<T: FromSudoOption>(&self, k: &str) -> Result<T> {
        let v = self.get(k)
            .chain_err(|| format!("option {} wasn't provided to the plugin", k) )?;

        FromSudoOption::from_sudo_option(v).ok()
            .chain_err(|| format!("option {} couldn't be parsed", k) )
    }

    pub fn get_raw(&self, k: &[u8]) -> Option<&[u8]> {
        self.0
            .get(k)
            .map(Vec::as_slice)
    }
}

pub trait FromSudoOption : Sized {
    type Err;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err>;
}

impl FromSudoOption for bool {
    type Err = ::std::str::ParseBoolError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for u16 {
    type Err = ::std::num::ParseIntError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for i32 {
    type Err = ::std::num::ParseIntError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for u32 {
    type Err = ::std::num::ParseIntError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for u64 {
    type Err = ::std::num::ParseIntError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl FromSudoOption for String {
    type Err = ::std::string::ParseError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        FromStr::from_str(s)
    }
}

impl<T> FromSudoOption for Vec<T> where T: FromSudoOption + FromSudoOptionList {
    type Err = ParseListError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let      list = <T as FromSudoOptionList>::from_sudo_option_list(s);
        let mut items = Vec::with_capacity(list.len());

        for element in list {
            let item = FromSudoOption::from_sudo_option(element)
                .map_err(|_| ParseListError() )?;

            items.push(item);
        }

        Ok(items)
    }
}

pub trait FromSudoOptionList : Sized {
    const SEPARATOR: char = ',';

    fn from_sudo_option_list(s: &str) -> Vec<&str> {
        s
            .split (|b| b == Self::SEPARATOR )
            .collect()
    }
}

impl FromSudoOptionList for i32 {}
impl FromSudoOptionList for u32 {}

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
