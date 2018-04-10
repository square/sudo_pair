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
    #[cfg_attr(feature="cargo-clippy", allow(cast_possible_truncation))]
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
