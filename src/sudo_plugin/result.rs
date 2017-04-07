use std::error::{self, Error as StdError};
use std::ffi;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    Conversation,
    Ffi(ffi::NulError),
    MissingKey(String, String),
}

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind: kind }
    }

    pub fn new_missing_key(key: &str, value: &str) -> Error {
        Error { kind: ErrorKind::MissingKey(
            key.to_string(), value.to_string(),
        ) }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Conversation             => self.description().fmt(fmt),
            ErrorKind::Ffi(ref e)               => e.fmt(fmt),
            ErrorKind::MissingKey(ref k, ref v) => write!(fmt, "{}[{}] missing", k, v),
        }
    }
}


impl StdError for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::Conversation   => "couldn't print output",
            ErrorKind::Ffi(ref e)     => e.description(),
            ErrorKind::MissingKey(..) => "configuration option missing",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self.kind {
            ErrorKind::Conversation   => None,
            ErrorKind::Ffi(ref e)     => e.cause(),
            ErrorKind::MissingKey(..) => None,
        }
    }
}

impl From<ffi::NulError> for Error {
    fn from(e: ffi::NulError) -> Self {
        Error { kind: ErrorKind::Ffi(e) }
    }
}
