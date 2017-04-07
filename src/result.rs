use sudo_plugin;

use std::error::{self, Error as StdError};
use std::fmt;
use std::io;
use std::num;
use std::result;

#[derive(Debug)]
pub enum Error {
    Unauthorized,
    Io(io::Error),
    Parse(num::ParseIntError),
    SudoPlugin(sudo_plugin::Error),
}

pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Unauthorized      => self.description().fmt(f),
            Error::Io(ref e)         => e.fmt(f),
            Error::Parse(ref e)      => e.fmt(f),
            Error::SudoPlugin(ref e) => e.fmt(f)
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Unauthorized      => "authorization declined",
            Error::Io(ref e)         => e.description(),
            Error::Parse(ref e)      => e.description(),
            Error::SudoPlugin(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Unauthorized      => None,
            Error::Io(ref e)         => e.cause(),
            Error::Parse(ref e)      => e.cause(),
            Error::SudoPlugin(ref e) => e.cause(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(e: num::ParseIntError) -> Self {
        Error::Parse(e)
    }
}

impl From<sudo_plugin::Error> for Error {
    fn from(e: sudo_plugin::Error) -> Self {
        Error::SudoPlugin(e)
    }
}
