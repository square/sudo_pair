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

use std::collections::HashMap;
use std::io::{Read, Result};
use std::fs::File;
use std::path::Path;

const DEFAULT_ESCAPE_BYTE : u8 = b'%';

pub(crate) struct Template {
    raw:    Vec<u8>,
    escape: u8,
}

impl Template {
    // TODO(rust 1.27) implement TryFrom when this trait stabilizes
    pub(crate) fn try_from<T: Read>(mut io: T) -> Result<Self> {
        let mut template = Vec::new();

        io.read_to_end(&mut template).map(|_| Self::from(template) )
    }

    pub(crate) fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        File::open(path).and_then(Self::try_from)
    }

    pub(crate) fn expand(&self, spec: &HashMap<u8, Vec<u8>>) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.raw.len());
        let mut iter   = self.raw.iter().cloned();

        while iter.len() != 0 {
            // copy literally everything up to the next escape character
            result.extend(
                iter.by_ref().take_while(|b| *b != self.escape )
            );

            // TODO: The above take_while consumes an extra byte in
            // the event that it finds the escape character; this is
            // *mostly* okay, but we can't distinguish between the case
            // where a '%' was consumed and where one wasn't (for
            // instance at EOF). This matters because of the following
            // line which terminates if there's nothing left in the
            // template to evaluate, because the template may have ended
            // in a '%'! This isn't a huge deal, but it's at least worth
            // documenting this limitation: if your template ends in a
            // line '%' character, we will silently eat it.
            let byte = match iter.next() {
                Some(b) => b,
                None    => break,
            };

            // if the spec contains an expansion for the escaped
            // character, use it; otherwise, emit the character as a
            // literal
            match spec.get(&byte) {
                Some(expansion) => result.extend_from_slice(expansion),
                None            => result.push(byte),
            };
        }

        result
    }
}

impl Default for Template {
    fn default() -> Self {
        Self {
            raw:    Vec::new(),
            escape: DEFAULT_ESCAPE_BYTE,
        }
    }
}

impl<'a> From<&'a [u8]> for Template {
    fn from(buf: &'a [u8]) -> Self {
        Self::from(buf.to_vec())
    }
}

impl From<Vec<u8>> for Template {
    fn from(buf: Vec<u8>) -> Self {
        Self { raw: buf, .. Self::default() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_template_from_buffer() {
        let raw      = b"simple template";
        let template = Template::from(&raw[..]);

        assert_eq!(raw[..], template.raw[..]);
    }

    #[test]
    fn new_template_default_escape_character() {
        let template = Template::from(&b""[..]);

        assert_eq!(DEFAULT_ESCAPE_BYTE, template.escape);
    }

    #[test]
    fn no_expansions() {
        let raw      = b"this has no expansions";
        let template = Template::from(&raw[..]);
        let spec     = HashMap::new();

        assert_eq!(raw[..], template.expand(&spec)[..]);
    }

    #[test]
    fn expansions() {
        let     raw      = b"a: %a, b: %b";
        let     template = Template::from(&raw[..]);
        let mut spec     = HashMap::new();

        let _ = spec.insert(b'a', b"foo".to_vec());
        let _ = spec.insert(b'b', b"bar".to_vec());

        assert_eq!(
            b"a: foo, b: bar"[..],
            template.expand(&spec)[..],
        );
    }

    #[test]
    fn repeated_expansions() {
        let     raw      = b"%a%a%a%b%a%a%b";
        let     template = Template::from(&raw[..]);
        let mut spec     = HashMap::new();

        let _ = spec.insert(b'a', b"x".to_vec());
        let _ = spec.insert(b'b', b"y".to_vec());

        assert_eq!(
            b"xxxyxxy"[..],
            template.expand(&spec)[..],
        );
    }

    #[test]
    fn expansion_inserts_itself() {
        let     raw      = b"test %x test";
        let     template = Template::from(&raw[..]);
        let mut spec     = HashMap::new();

        let _ = spec.insert(b'x', b"x".to_vec());

        assert_eq!(
            b"test x test"[..],
            template.expand(&spec)[..],
        );
    }

    #[test]
    fn expansion_isnt_recursive() {
        let     raw      = b"test %x test";
        let     template = Template::from(&raw[..]);
        let mut spec     = HashMap::new();

        let _ = spec.insert(b'x', b"%x %y %z % %%".to_vec());
        let _ = spec.insert(b'y', b"BUG".to_vec());

        assert_eq!(
            b"test %x %y %z % %% test"[..],
            template.expand(&spec)[..],
        );
    }

    #[test]
    fn expansion_inserts_nothing() {
        let     raw      = b"test %X test";
        let     template = Template::from(&raw[..]);
        let mut spec     = HashMap::new();

        let _ = spec.insert(b'X', b"".to_vec());

        assert_eq!(
            b"test  test"[..],
            template.expand(&spec)[..],
        );
    }

    #[test]
    fn unused_expansions() {
        let     raw      = b"only y should be expanded %y";
        let     template = Template::from(&raw[..]);
        let mut spec     = HashMap::new();

        let _ = spec.insert(b'y', b"qwerty".to_vec());
        let _ = spec.insert(b'n', b"uiop[".to_vec());

        assert_eq!(
            b"only y should be expanded qwerty"[..],
            template.expand(&spec)[..],
        );
    }

    #[test]
    fn literals() {
        let     raw      = b"a: %a, b: %b";
        let     template = Template::from(&raw[..]);
        let mut spec     = HashMap::new();

        let _ = spec.insert(b'b', b"bar".to_vec());

        assert_eq!(
            b"a: a, b: bar"[..],
            template.expand(&spec)[..],
        );
    }

    #[test]
    fn literal_escape_character() {
        let raw      = b"%%%%%%%%%%%%%%";
        let template = Template::from(&raw[..]);
        let spec     = HashMap::new();

        assert_eq!(
            b"%%%%%%%"[..],
            template.expand(&spec)[..],
        );
    }

    // you can currently provide an expansion for the escape character,
    // which prevents ever being able to insert an escape character
    // literal; this isn't worth fixing
    #[test]
    fn bug_wontfix_expand_escape_character() {
        let     raw      = b"|%%|";
        let     template = Template::from(&raw[..]);
        let mut spec     = HashMap::new();

        let _ = spec.insert(b'%', b"x".to_vec());

        assert_eq!(b"|x|"[..], template.expand(&spec)[..]);
    }

    // `take_while` silently eats one extra character off of the `Iter`,
    // since it needs to call `Iter::next` before it can check the value,
    // and this leads us to not being able to detect the difference
    // between a normal EOF and an EOF immediately after a lone escape
    // character; this isn't worth fixing
    #[test]
    fn bug_wontfix_swallow_trailing_escape_character() {
        let raw      = b"some text%";
        let template = Template::from(&raw[..]);
        let spec     = HashMap::new();

        assert_eq!(b"some text"[..], template.expand(&spec)[..]);
    }
}
