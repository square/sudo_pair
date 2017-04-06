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
    MissingSetting(SettingKind, &'static str),
}

#[derive(Debug)]
pub enum SettingKind {
    // Settings,
    UserInfo,
    CommandInfo,
    // UserEnv,
}

pub type Result<T> = result::Result<T, Error>;

impl SettingKind {
    fn as_str(&self) -> &'static str {
        match *self {
            // SettingKind::Settings    => "settings",
            SettingKind::UserInfo    => "user_info",
            SettingKind::CommandInfo => "command_info",
            // SettingKind::UserEnv     => "user_env",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Unauthorized => self.description().fmt(f),
            Error::Io(ref e)    => e.fmt(f),
            Error::Parse(ref e) => e.fmt(f),
            Error::MissingSetting(ref k, ref n) =>
                write!(f, "sudo_plugin {} missing setting {}", k.as_str(), n),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Unauthorized        => "authorization declined",
            Error::Io(ref e)           => e.description(),
            Error::Parse(ref e)        => e.description(),
            Error::MissingSetting(_, _) => "sudo_plugin missing expected setting",
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
