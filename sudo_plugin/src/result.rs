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

#![allow(box_pointers)]
#![allow(missing_copy_implementations)]
#![cfg_attr(feature="clippy", allow(match_same_arms))]

use std::convert::From;
use std::error;
use std::io;
use std::ffi;
use std::fmt;
use std::num;
use std::result;

use libc::c_int;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Ffi(ffi::NulError),
    Io(io::Error),
    Simple(ErrorKind),
    Custom(ErrorKind, Box<error::Error + Send + Sync>),
}

#[derive(Debug)]
pub enum ErrorKind {
    // An option expected to be provided to the plugin was missing.
    MissingOption,

    // The plugin was not initialized properly.
    Uninitialized,

    // The command is not authorized.
    Unauthorized,

    // Unspecified error.
    Other,
}

pub trait AsSudoPluginRetval {
    fn as_sudo_plugin_retval(&self) -> c_int;
}

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
        where E: Into<Box<error::Error + Send + Sync>>
    {
        Error::Custom(kind, error.into())
    }
}

impl ErrorKind {
    fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::MissingOption       => "expected an option that wasn't present",
            ErrorKind::Uninitialized       => "the plugin failed to initialize",
            ErrorKind::Unauthorized        => "command unauthorized",
            ErrorKind::Other               => "unknown error",
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Ffi(ref e)       => e.description(),
            Error::Io(ref e)        => e.description(),
            Error::Simple(ref k)    => k.as_str(),
            Error::Custom(ref k, _) => k.as_str(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Ffi(ref e)       => e.cause(),
            Error::Io(ref e)        => e.cause(),
            Error::Simple(_)        => None,
            Error::Custom(_, ref e) => e.cause(),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error::Simple(kind)
    }
}

impl From<ffi::NulError> for Error {
    fn from(err: ffi::NulError) -> Self {
        Error::Ffi(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

// TODO: this one doesn't really belong here :(
impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Self {
        Error::Custom(ErrorKind::Other, err.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Ffi(ref e)       => e.fmt(fmt),
            Error::Io(ref e)        => e.fmt(fmt),
            Error::Simple(ref k)    => write!(fmt, "{}", k.as_str()),
            Error::Custom(_, ref e) => e.fmt(fmt),
        }
    }
}

impl<T> AsSudoPluginRetval for Result<T> {
    fn as_sudo_plugin_retval(&self) -> c_int {
        match *self {
            Ok(_)                                          =>  1,
            Err(Error::Simple(ErrorKind::Unauthorized))    =>  0,
            Err(Error::Custom(ErrorKind::Unauthorized, _)) =>  0,
            Err(_)                                         => -1,
        }
    }
}
