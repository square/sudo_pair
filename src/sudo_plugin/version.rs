use libc::c_uint;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Version {
    major: u16,
    minor: u16,
}

impl From<c_uint> for Version {
    fn from(version: c_uint) -> Self {
        Version{
            major: (version >> 16)     as u16,
            minor: (version &  0xffff) as u16,
        }
    }
}
