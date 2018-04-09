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

use super::errors::*;

use std::fmt;

use libc::c_uint;

const MINIMUM_MAJOR: u16 = 1;
const MINIMUM_MINOR: u16 = 9;

const MINIMUM: Version = Version {
    major: MINIMUM_MAJOR,
    minor: MINIMUM_MINOR,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Version {
    major: u16,
    minor: u16,
}

impl Version {
    pub fn minimum() -> &'static Self {
        &MINIMUM
    }

    pub fn supported(&self) -> bool {
        self >= Self::minimum()
    }

    pub fn check(self) -> Result<Self> {
        if !self.supported() {
            bail!(ErrorKind::UnsupportedApiVersion(self))
        }

        Ok(self)
    }
}

impl From<c_uint> for Version {
    #[cfg_attr(test, allow(cast_possible_truncation))]
    fn from(version: c_uint) -> Self {
        Self {
            major: (version >> 16)     as _,
            minor: (version &  0xffff) as _,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
