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

// TODO: remove when error_chain is updated to compile cleanly
#![allow(bare_trait_objects)]
#![allow(renamed_and_removed_lints)]
#![allow(single_use_lifetimes)]
#![allow(variant_size_differences)]

use crate::version::Version;

use std::result::Result as StdResult;
use std::error::Error as StdError;
use thiserror::Error;

/// Return codes understood by the `io_plugin.open` callback.
///
/// The interpretations of these values are badly-documented within the
/// [`sudo_plugin(8)` manpage][manpage] so the code was used to
/// understand their actual effects.
///
/// [manpage]: https://www.sudo.ws/man/1.8.30/sudo_plugin.man.html
/// [code]: https://github.com/sudo-project/sudo/blob/446ae3f507271c8a08f054c9291cb8804afe81d9/src/sudo.c#L1404
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum OpenStatus {
    /// The plugin was `open`ed successfully and may be used as normal.
    Ok = 1,

    /// The plugin should be unloaded for the duration of this `sudo`
    /// session. The `sudo` session may continue, but will not use any
    /// of the features of this plugin.
    Disable = 0,

    /// The `sudo` command is unauthorized and must be immediately
    /// terminated.
    Deny = -1,

    /// The `sudo` command was invoked incorrectly and will be
    /// terminated. Basic usage information will be presented to the
    /// user. The plugin may choose to emit its own usage information
    /// describing the problem.
    Usage = -2,
}

/// Return codes understood by the `io_plugin.log_*` family of callbacks.
///
/// The interpretations of these values are badly-documented within the
/// [`sudo_plugin(8)` manpage][manpage] so the code was used to
/// understand their actual effects.
///
/// [manpage]: https://www.sudo.ws/man/1.8.30/sudo_plugin.man.html
/// [code]: https://github.com/sudo-project/sudo/blob/446ae3f507271c8a08f054c9291cb8804afe81d9/src/sudo.c#L1404
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum LogStatus {
    /// The plugin logged the information successfully.
    Ok = 1,

    /// The plugin has determined that the `sudo` session should be
    /// terminated immediately.
    Deny = 0,

    /// The plugin no longer needs this callback. This callback will no
    /// longer be invoked by `sudo`, but the rest of the plugin's
    /// callbacks will function as normal.
    Disable = -1,
}

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
        key:   String,

        /// The value provided.
        value: String,
    },

    /// A generic error identified only by a provided string. This may
    /// be used by plugin implementors who don't wish to provide their
    /// own custom error types, and instead are happy to simply use
    /// stringly-typed error messages.
    #[error("plugin exited: {0}")]
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

impl<T, E: SudoError> From<StdResult<T, E>> for OpenStatus {
    fn from(result: StdResult<T, E>) -> Self {
        match result {
            Ok(_)  => OpenStatus::Ok,
            Err(e) => e.into(),
        }
    }
}

impl<T, E: SudoError> From<StdResult<T, E>> for LogStatus {
    fn from(result: StdResult<T, E>) -> Self {
        match result {
            Ok(_)  => LogStatus::Ok,
            Err(e) => e.into(),
        }
    }
}
