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

use super::SudoPair;

use std::convert::From;
use std::io::{Read, Result};
use std::fs::File;
use std::path::Path;
use std::os::unix::ffi::OsStrExt;

const TEMPLATE_ESCAPE : u8 = b'%';

pub(crate) struct Prompt {
    template: Vec<u8>,
}

// TODO: document the semantics of this in the README
impl Prompt {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self { template: Vec::with_capacity(capacity) }
    }

    pub(crate) fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let     len  = file.metadata().map(|m| m.len() )?;

        // TODO(rust 1.27): replace cast with TryFrom when it's
        // stabilized; for now, this seems safe to ignore since wrapping
        // would just cause us to preallocate an array smaller than
        // anticipated (which would be +4.7GB anyway...)
        #[cfg_attr(feature="cargo-clippy", allow(cast_possible_truncation))]
        let mut prompt = Self::with_capacity(len as usize);

        let _ = file.read_to_end(&mut prompt.template);

        Ok(prompt)
    }

    pub(crate) fn expand(&self, pair: &SudoPair) -> Vec<u8> {
        let mut result = vec![];
        let mut iter   = self.template.iter().cloned();

        while iter.len() != 0 {
            // copy everything up to the next %-sign unchanged
            result.extend(
                iter.by_ref().take_while(|b| *b != TEMPLATE_ESCAPE )
            );

            // if there's nothing left, we're done
            if iter.len() == 0 {
                break;
            }

            // we expand each literal into an owned type so that we don't have
            // to repeatd the `result.extend_from_slice` part each time in the
            // match arms, but it does kind of suck that we have so much
            // type-conversion noise
            //
            // TODO: document these somewhere useful for users of this plugin
            // TODO: provide groupname of gid?
            // TODO: provide username of runas_euid?
            // TODO: provide groupname of runas_egid?
            let expansion = match iter.next() {
                // the name of the appoval _b_inary
                Some(b'b') => pair.options.binary_name().into(),

                // the full path to the approval _B_inary
                Some(b'B') => pair.options.binary_path.as_os_str().as_bytes().into(),

                // the full _C_ommand `sudo` was invoked as (recreated as
                // best-effort for now)
                Some(b'C') => pair.plugin.invocation(),

                // the cw_d_ of the command being run under `sudo`
                Some(b'd') => pair.plugin.cwd().as_os_str().as_bytes().into(),

                // the _h_ostname of the machine `sudo` is being executed on
                Some(b'h') => pair.plugin.user_info.host.as_bytes().into(),

                // the _H_eight of the invoking user's terminal, in rows
                Some(b'H') => pair.plugin.user_info.lines.to_string().into_bytes(),

                // the real _g_id of the user invoking `sudo`
                Some(b'g') => pair.plugin.user_info.gid.to_string().into_bytes(),

                // the _p_id of this `sudo` process
                Some(b'p') => pair.plugin.user_info.pid.to_string().into_bytes(),

                // the real _u_id of the user invoking `sudo`
                Some(b'u') => pair.plugin.user_info.uid.to_string().into_bytes(),

                // the _U_sername of the user running `sudo`
                Some(b'U') => pair.plugin.user_info.user.as_bytes().into(),

                // the _W_idth of the invoking user's terminal, in columns
                Some(b'W') => pair.plugin.user_info.cols.to_string().into_bytes(),

                Some(byte) => vec![TEMPLATE_ESCAPE, byte],
                None       => vec![TEMPLATE_ESCAPE],
            };

            result.extend_from_slice(&expansion[..]);
        }

        result
    }
}

impl<'a> From<&'a [u8]> for Prompt {
    fn from(template: &'a [u8]) -> Self {
        Self { template: template.to_vec() }
    }
}
