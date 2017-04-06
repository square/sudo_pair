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
