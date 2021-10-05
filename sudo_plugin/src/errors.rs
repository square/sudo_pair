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

//! The collection of `Error` types used by this library.

// TODO: use error types as directly defined by sudo_plugin(8).

use crate::core::{OpenStatus, LogStatus};
use crate::version::Version;

use std::result::Result as StdResult;
use std::error::Error as StdError;
use thiserror::Error;

/// Errors that can be produced by plugin internals.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum Error {
    /// The plugin was called using the conventions of an unsupported
    /// API version.
    #[error("sudo called plugin with an API version of {provided}, but a minimum of {required} is required")]
    UnsupportedApiVersion {
        /// The minimum API version supported
        required: Version,

        /// The API version provided by `sudo`.
        provided: Version,
    },

    /// A required option is missing.
    #[error("sudo called plugin without providing a value for {key}")]
    OptionMissing {
        /// The name of the option.
        key: String,
    },

    /// A required option could not be parsed into the expected type.
    #[error("sudo called plugin with an unparseable value for {key}: {value}")]
    OptionInvalid {
        /// The name of the option.
        key: String,

        /// The value provided.
        value: String,
    },

    /// A plugin method panicked and the panic was captured at the FFI
    /// boundary. Panics can't cross into C, so we have to capture it
    /// and turn it into an appropriate return code.
    #[error("uncaught internal error")]
    UncaughtPanic,

    /// A generic error identified only by a provided string. This may
    /// be used by plugin implementors who don't wish to provide their
    /// own custom error types, and instead are happy to simply use
    /// stringly-typed error messages.
    #[error("{0}")]
    Other(String),
}

pub(crate) type Result<T> = StdResult<T, Error>;

/// The type for errors that can be returned from plugin callbacks.
/// Plugin authors are expected to provide an implementation of coercion
/// `From<sudo_plugin::errors::Error>` for their own custom error types
/// as well coercions `Into<OpenStatus>` and `Into<LogStatus>` to
/// specify how those errors should be treated by `sudo`.
pub trait SudoError: StdError + From<Error> + Into<OpenStatus> + Into<LogStatus> { }

impl<T: StdError + From<Error> + Into<OpenStatus> + Into<LogStatus>> SudoError for T {}

impl From<Error> for OpenStatus {
    fn from(_: Error) -> Self {
        // by default, abort `sudo` on all errors
        OpenStatus::Deny
    }
}

impl From<Error> for LogStatus {
    fn from(_: Error) -> Self {
        // by default, abort `sudo` on all errors
        LogStatus::Deny
    }
}
