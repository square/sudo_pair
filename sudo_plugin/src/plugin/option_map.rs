use super::super::errors::*;

use std::collections::HashMap;
use std::ffi::CStr;
use std::path::PathBuf;
use std::str::{self, FromStr};

use libc::c_char;

const OPTIONS_SEPARATOR: u8 = b'=';

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
    /// provided when `sudo` invokes the plugin's entry function. The
    /// format of these is a NUL-terminated array of NUL-terminated
    /// strings in "key=value" format with the array terminated by a
    /// NULL pointer.
    ///
    /// This method cannot be safe, since it relies on the caller to
    /// place a NULL byte as the final array entry. In the absence of
    /// such a NULL byte, there is no other way to detect the end of
    /// the options list.
    pub unsafe fn new(mut ptr: *const *const c_char) -> Self {
        // if the pointer is null, we weren't given a list of settings,
        // so go ahead and return the empty map
        if ptr.is_null() {
            return Self::default();
        }

        let mut map = HashMap::new();

        // iterate through each pointer in the array until encountering
        // a NULL (which terminates the array)
        while !(*ptr).is_null() {
            let bytes = CStr::from_ptr(*ptr).to_bytes();
            let sep = bytes.iter().position(|b| *b == OPTIONS_SEPARATOR);

            // separators might not exist (e.g., in the case of parsing
            // plugin options; for this case, we use the full entry as
            // both the name of the key and its value
            let (k, v) = match sep {
                Some(s) => { ( &bytes[..s], &bytes[s+1..] ) }
                None    => { ( &bytes[..],  &bytes[..] ) }
            };

            // the return value is ignored because there's not an
            // otherwise-reasonable way to handle duplicate key names;
            // that said, the implications of this are that the last
            // value of a given key is the one that's set
            let _ = map.insert(
                k.to_owned(),
                v.to_owned(),
            );

            ptr = ptr.offset(1);
        }

        OptionMap(map)
    }

    /// Gets the value of a key as any arbitrary type that implements the
    /// `FromSudoOption` trait. Returns `Err(_)` if no such key/value-pair
    /// was provided during initialization. Also returns `Err(_)` if the
    /// value was not interpretable as a UTF-8 string or if there was an
    /// error parsing the value to the requested type.
    pub fn get<T: FromSudoOption>(&self, k: &str) -> Result<T> {
        let v = self.get_str(k).chain_err(|| {
            format!("option {} wasn't provided to the plugin", k)
        })?;

        FromSudoOption::from_sudo_option(v)
            .ok()
            .chain_err(|| format!("option {} couldn't be parsed", k))
    }

    /// Gets the value of a key as a string. Returns `None` if no such
    /// key/value-pair was provided during initialization. Also returns
    /// `None` if the value was not interpretable as a UTF-8 string.
    pub fn get_str(&self, k: &str) -> Option<&str> {
        self.get_bytes(k.as_bytes()).and_then(|b| str::from_utf8(b).ok())
    }

    /// Fetches a raw byte value using a bytes as the key. This is
    /// provided to allow plugins to retrieve values for keys when the
    /// value and/or key are not guaranteed to be UTF-8 strings.
    pub fn get_bytes(&self, k: &[u8]) -> Option<&[u8]> {
        self.0.get(k).map(Vec::as_slice)
    }
}

impl Default for OptionMap {
    fn default() -> OptionMap {
        OptionMap(HashMap::new())
    }
}

pub trait FromSudoOption: Sized {
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

impl<T> FromSudoOption for Vec<T>
where
    T: FromSudoOption + FromSudoOptionList,
{
    type Err = ParseListError;

    fn from_sudo_option(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let      list = <T as FromSudoOptionList>::from_sudo_option_list(s);
        let mut items = Self::with_capacity(list.len());

        for element in list {
            let item = FromSudoOption::from_sudo_option(element)
                .map_err(|_| ParseListError())?;

            items.push(item);
        }

        Ok(items)
    }
}

pub trait FromSudoOptionList: Sized {
    const SEPARATOR: char = ',';

    fn from_sudo_option_list(s: &str) -> Vec<&str> {
        s.split(|b| b == Self::SEPARATOR).collect()
    }
}

impl FromSudoOptionList for i32 {}
impl FromSudoOptionList for u32 {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn new_parses_simple_keys() {
        let map = unsafe { OptionMap::new([
            b"key1=value1\0".as_ptr() as _,
            b"key2=value2\0".as_ptr() as _,
            ptr::null(),
        ].as_ptr()) };

        assert_eq!(Some("value1"), map.get_str("key1"));
        assert_eq!(Some("value2"), map.get_str("key2"));
        assert_eq!(None,           map.get_str("key3"));
    }
}
