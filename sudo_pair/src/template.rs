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

const DEFAULT_ESCAPE_BYTE : u8 = b'%';

pub(crate) struct Spec {
    escape:     u8,
    expansions: HashMap<u8, Vec<u8>>,
}

impl Spec {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_escape(escape: u8) -> Self {
        Self { escape, .. Self::new() }
    }

    pub(crate) fn replace<T: Into<Vec<u8>>>(&mut self, literal: u8, replacement: T) {
        drop(self.expansions.insert(literal, replacement.into()));
    }

    pub(crate) fn expand(&self, template: &[u8]) -> Vec<u8> {
        // the expanded result is likely to be at least as long as the
        // template; if we go a little over, it's not a big deal
        let mut result = Vec::with_capacity(template.len());
        let mut iter   = template.iter().copied();

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
            match self.expansions.get(&byte) {
                Some(expansion) => result.extend_from_slice(expansion),
                None            => result.push(byte),
            };
        }

        result
    }
}

impl Default for Spec {
    fn default() -> Self {
        Self {
            expansions: HashMap::new(),
            escape:     DEFAULT_ESCAPE_BYTE,
        }
    }
}

impl From<HashMap<u8, Vec<u8>>> for Spec {
    fn from(expansions: HashMap<u8, Vec<u8>>) -> Self {
        Self { expansions, .. Self::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let spec = Spec::new();

        assert_eq!(DEFAULT_ESCAPE_BYTE, spec.escape);
    }

    #[test]
    fn with_escape() {
        let spec = Spec::with_escape(b'\\');

        assert_eq!(b'\\', spec.escape);
    }

    #[test]
    fn from_hashmap() {
        let mut map = HashMap::new();
        let _       = map.insert(b'x', b"abc".to_vec());

        let spec = Spec::from(map.clone());

        assert_eq!(map, spec.expansions);
    }

    #[test]
    fn no_expansions() {
        let spec     = Spec::new();
        let template = b"this has no expansions";

        assert_eq!(
            template[..],
            spec.expand(template)[..]
        );
    }

    #[test]
    fn expansions() {
        let mut spec     = Spec::new();
        let     template = b"a: %a, b: %b";

        let _ = spec.replace(b'a', &b"foo"[..]);
        let _ = spec.replace(b'b', &b"bar"[..]);

        assert_eq!(
            b"a: foo, b: bar"[..],
            spec.expand(template)[..],
        );
    }

    #[test]
    fn repeated_expansions() {
        let mut spec     = Spec::new();
        let     template = b"%a%a%a%b%a%a%b";

        let _ = spec.replace(b'a', &b"x"[..]);
        let _ = spec.replace(b'b', &b"y"[..]);

        assert_eq!(
            b"xxxyxxy"[..],
            spec.expand(template)[..],
        );
    }

    #[test]
    fn expansion_inserts_itself() {
        let mut spec     = Spec::new();
        let     template = b"test %x test";

        spec.replace(b'x', &b"x"[..]);

        assert_eq!(
            b"test x test"[..],
            spec.expand(template)[..],
        );
    }

    #[test]
    fn expansion_isnt_recursive() {
        let mut spec     = Spec::new();
        let     template = b"test %x test";

        spec.replace(b'x', &b"%x %y %z % %%"[..]);
        spec.replace(b'y', &b"BUG"[..]);

        assert_eq!(
            b"test %x %y %z % %% test"[..],
            spec.expand(template)[..],
        );
    }

    #[test]
    fn expansion_inserts_nothing() {
        let mut spec     = Spec::new();
        let     template = b"test %X test";

        spec.replace(b'X', &b""[..]);

        assert_eq!(
            b"test  test"[..],
            spec.expand(template)[..],
        );
    }

    #[test]
    fn unused_expansions() {
        let mut spec     = Spec::new();
        let     template = b"only y should be expanded %y";

        spec.replace(b'y', &b"qwerty"[..]);
        spec.replace(b'n', &b"uiop["[..]);

        assert_eq!(
            b"only y should be expanded qwerty"[..],
            spec.expand(template)[..],
        );
    }

    #[test]
    fn literals() {
        let mut spec     = Spec::new();
        let     template = b"a: %a, b: %b";

        spec.replace(b'b', &b"bar"[..]);

        assert_eq!(
            b"a: a, b: bar"[..],
            spec.expand(template)[..],
        );
    }

    #[test]
    fn literal_escape_character() {
        let spec     = Spec::new();
        let template = b"%%%%%%%%%%%%%%";

        assert_eq!(
            b"%%%%%%%"[..],
            spec.expand(template)[..],
        );
    }

    // you can currently provide an expansion for the escape character,
    // which prevents ever being able to insert an escape character
    // literal; this isn't worth fixing
    #[test]
    fn bug_wontfix_expand_escape_character() {
        let mut spec     = Spec::new();
        let     template = b"|%%|";

        spec.replace(b'%', &b"x"[..]);

        assert_eq!(
            b"|x|"[..],
            spec.expand(template)[..]
        );
    }

    // `take_while` silently eats one extra character off of the `Iter`,
    // since it needs to call `Iter::next` before it can check the value,
    // and this leads us to not being able to detect the difference
    // between a normal EOF and an EOF immediately after a lone escape
    // character; this isn't worth fixing
    #[test]
    fn bug_wontfix_swallow_trailing_escape_character() {
        let spec     = Spec::new();
        let template = b"some text%";

        assert_eq!(
            b"some text"[..],
            spec.expand(template)[..]
        );
    }
}
