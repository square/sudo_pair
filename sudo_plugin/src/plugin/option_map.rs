use super::super::errors::*;

use std::collections::HashMap;
use std::ffi::CStr;
use std::path::PathBuf;
use std::str::{self, FromStr};

use libc::c_char;

const OPTIONS_SEPARATOR : u8 = b'=';

/// A HashMap-like list of options parsed from the pointers provided by
/// the underlying sudo plugin API.
///
/// Allows for automatic parsing of values into any type which implements
/// the `FromSudoOption` trait as well as values into a `Vec` of any type
/// which implements the `FromSudoOptionList` trait.
#[derive(Clone, Debug)]
pub struct OptionMap(HashMap<Vec<u8>, Vec<u8>>);

#[derive(Clone, Copy, Debug)]
pub struct ParseListError();

impl OptionMap {
    /// Initializes the `OptionMap` from a pointer to the options
    /// provided when `sudo` invokes the plugin's entry function.
    ///
    /// The format of these is a NUL-terminated array of NUL-terminated
    /// strings in "key=value" format.
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

        Ok(OptionMap(map))
    }

    /// Gets the value of a key as a string. Returns `None` if no such
    /// key/value-pair was provided during initialization. Also returns
    /// `None` if the value was not interpretable as a UTF-8 string.
    pub fn get(&self, k: &str) -> Option<&str> {
        self.get_raw(k.as_bytes())
            .and_then(|b| str::from_utf8(b).ok() )
    }

    /// Gets the value of a key as any arbitrary type that implements the
    /// `FromSudoOption` trait. Returns `Err(_)` if no such key/value-pair
    /// was provided during initialization. Also returns `None` if the
    /// value was not interpretable as a UTF-8 string or if there was an
    /// error parsing the value to the requested type.
    pub fn get_parsed<T: FromSudoOption>(&self, k: &str) -> Result<T> {
        let v = self.get(k)
            .chain_err(|| format!("option {} wasn't provided to the plugin", k) )?;

        FromSudoOption::from_sudo_option(v).ok()
            .chain_err(|| format!("option {} couldn't be parsed", k) )
    }

    /// Fetches a raw byte value using a bytes as the key. This is
    /// provided to allow plugins to retrieve values for keys when the
    /// value and/or key are not guaranteed to be UTF-8 strings.
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

impl FromSudoOption for PathBuf {
    type Err = ::std::string::ParseError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(s.into())
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
