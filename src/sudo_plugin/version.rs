use libc::c_uint;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Version {
    major: u16,
    minor: u16,
}

impl From<c_uint> for Version {
    #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
    fn from(version: c_uint) -> Self {
        Self {
            major: (version >> 16)     as _,
            minor: (version &  0xffff) as _,
        }
    }
}
