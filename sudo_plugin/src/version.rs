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

use crate::errors::{Result, Error};
use crate::sys;

use std::fmt;
use std::os::raw::c_uint;

const MINIMUM: Version = Version::new(1, 9);

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Version {
    major: u16,
    minor: u16,
}

impl Version {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    pub const fn from_ffi(version: c_uint) -> Self {
        // this cast is guaranteed not to truncate thanks to the shifts
        // and masks
        #[allow(clippy::cast_possible_truncation)]
        Self::new(
            sys::sudo_api_version_get_major(version) as _,
            sys::sudo_api_version_get_minor(version) as _,
        )
    }

    pub const fn into_ffi(self) -> c_uint {
        sys::sudo_api_mkversion(
            self.major as _,
            self.minor as _,
        )
    }

    pub const fn minimum() -> &'static Self {
        &MINIMUM
    }

    pub fn supported(self) -> bool {
        self >= *Self::minimum()
    }

    pub fn check(self) -> Result<Self> {
        if !self.supported() {
            return Err(Error::UnsupportedApiVersion {
                required: MINIMUM,
                provided: self,
            });
        }

        Ok(self)
    }
}

impl From<c_uint> for Version {
    fn from(version: c_uint) -> Self {
        Self::from_ffi(version)
    }
}

impl From<Version> for c_uint {
    fn from(version: Version) -> Self {
        version.into_ffi()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
