use libc::c_uint;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Version {
    major: u16,
    minor: u16,
}

impl From<c_uint> for Version {
    fn from(version: c_uint) -> Self {
        Self {
            major: (version >> 16)     as _,
            minor: (version &  0xffff) as _,
        }
    }
}
