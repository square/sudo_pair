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

const DEFAULT_ESCAPE : u8 = b'%';

pub(crate) struct Template {
    template: Vec<u8>,
    escape:   u8,
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
        let mut result = Vec::with_capacity(self.template.len());
        let mut iter   = self.template.iter().cloned();

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
            template: Vec::new(),
            escape:   DEFAULT_ESCAPE,
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
        Self { template: buf, .. Self::default() }
    }
}
