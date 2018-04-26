use super::super::errors::*;

use super::traits::*;

use std::collections::HashMap;
use std::ffi::CStr;
use std::str;

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

// TOOD: in policy plugins, some of these values can be written back to
// by the plugin in order to change the execution of sudo itself (e.g.,
// `command_info.chroot`, `command_info.cwd`, and others). We should
// support that use-case, but I don't think the current design of the
// API can be plausibly made to support it. This will probably require
// a redesign of this entire thing, which I'm not really excited about.
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
    pub unsafe fn from_raw(mut ptr: *const *const c_char) -> Self {
        let mut map = HashMap::new();

        // if the pointer is null, we weren't given a list of settings,
        // so go ahead and return the empty map
        if ptr.is_null() {
            return OptionMap(map);
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::ptr;

    #[test]
    fn new_parses_string_keys() {
        let map = unsafe { OptionMap::from_raw([
            b"key1=value1\0".as_ptr() as _,
            b"key2=value2\0".as_ptr() as _,
            ptr::null(),
        ].as_ptr()) };

        assert_eq!("value1", map.get_str("key1").unwrap());
        assert_eq!("value2", map.get_str("key2").unwrap());
        assert!(map.get_str("key3").is_none());
    }

    #[test]
    fn new_parses_null_options() {
        let map = unsafe { OptionMap::from_raw(ptr::null()) };

        assert!(map.0.is_empty())
    }

    #[test]
    fn new_parses_non_utf8_keys() {
        let map = unsafe { OptionMap::from_raw([
            b"\x80=value\0".as_ptr() as _,
            ptr::null(),
        ].as_ptr()) };

        assert_eq!(&b"value"[..], map.get_bytes(b"\x80").unwrap());
    }

    #[test]
    fn new_parses_non_utf8_values() {
        let map = unsafe { OptionMap::from_raw([
            b"key=\x80\0".as_ptr() as _,
            ptr::null(),
        ].as_ptr()) };

        assert_eq!(None,         map.get_str("key"));
        assert_eq!(&b"\x80"[..], map.get_bytes(b"key").unwrap());
    }

    #[test]
    fn new_parses_repeated_keys() {
        let map = unsafe { OptionMap::from_raw([
            b"key=value1\0".as_ptr() as _,
            b"key=value2\0".as_ptr() as _,
            b"key=value3\0".as_ptr() as _,
            ptr::null(),
        ].as_ptr()) };

        assert_eq!("value3", map.get_str("key").unwrap());
    }

    #[test]
    fn new_parses_valueless_keys() {
        let map = unsafe { OptionMap::from_raw([
            b"key\0".as_ptr() as _,
            ptr::null(),
        ].as_ptr()) };

        assert_eq!("key", map.get_str("key").unwrap());
    }

    #[test]
    fn new_parses_values_with_the_separator() {
        let map = unsafe { OptionMap::from_raw([
            b"key=value=value\0".as_ptr() as _,
            ptr::null(),
        ].as_ptr()) };

        assert_eq!("value=value", map.get_str("key").unwrap());
        assert_eq!(None,          map.get_str("key=value"));
    }

    #[test]
    fn get_parses_common_types() {
        let map = unsafe { OptionMap::from_raw([
            b"str=value\0"    .as_ptr() as _,
            b"true\0"         .as_ptr() as _,
            b"false\0"        .as_ptr() as _,
            b"i64=-42\0"      .as_ptr() as _,
            b"u64=42\0"       .as_ptr() as _,
            b"path=/foo/bar\0".as_ptr() as _,
            ptr::null(),
        ].as_ptr()) };

        assert_eq!(String::from("value"),     map.get::<String>("str").unwrap());
        assert_eq!(PathBuf::from("/foo/bar"), map.get::<PathBuf>("path").unwrap());

        assert_eq!(true,  map.get::<bool>("true") .unwrap());
        assert_eq!(false, map.get::<bool>("false").unwrap());
        assert!(map.get::<bool>("str").is_err());

        assert_eq!(-42, map.get::<i8>("i64") .unwrap());
        assert_eq!(-42, map.get::<i16>("i64").unwrap());
        assert_eq!(-42, map.get::<i32>("i64").unwrap());
        assert_eq!(-42, map.get::<i64>("i64").unwrap());
        assert!(map.get::<i64>("true").is_err());

        assert_eq!(42,  map.get::<u8>("u64") .unwrap());
        assert_eq!(42,  map.get::<u16>("u64").unwrap());
        assert_eq!(42,  map.get::<u32>("u64").unwrap());
        assert_eq!(42,  map.get::<u64>("u64").unwrap());
        assert!(map.get::<u64>("i64").is_err());
    }

    #[test]
    fn get_parses_lists() {
        impl FromSudoOptionList for String {
            const SEPARATOR: char = '|';
        }

        let map = unsafe { OptionMap::from_raw([
            b"ints=1,2,3\0".as_ptr() as _,
            b"strs=a|b|c\0".as_ptr() as _,
            b"str=a,b,c\0" .as_ptr() as _,
            ptr::null(),
        ].as_ptr()) };

        assert_eq!(vec![1, 2, 3],       map.get::<Vec<u8>>("ints")    .unwrap());
        assert_eq!(vec!["a", "b", "c"], map.get::<Vec<String>>("strs").unwrap());
        assert_eq!(vec!["a,b,c"],       map.get::<Vec<String>>("str") .unwrap());
    }
}
