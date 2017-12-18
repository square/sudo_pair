//! The collection of `Error` types used by this library.

use super::version::Version;

use std::fmt;

use libc::c_int;

/// The list of supported facilities to communicate with the end-user.
#[derive(Clone, Copy, Debug)]
pub enum IoFacility {
    /// A printf-style function that can be used for one-way communication
    /// with the invoking user.
    PluginPrintf,

    /// A more complicated facility that enables two-way communication
    /// with the invoking user.
    Conversation,
}

impl fmt::Display for IoFacility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IoFacility::PluginPrintf => write!(f, "plugin_printf"),
            IoFacility::Conversation => write!(f, "conversation"),
        }
    }
}

error_chain! {
    errors {
        /// An error which can be returned when an option provided to the
        /// plugin cannot be parsed to the required type.
        ParseFailure(name: String) {
            description("sudo plugin was invoked with malformed options"),
            display("sudo plugin was invoked with a malformed {}", name),
        }

        /// An error which can be returned when performing I/O to the
        /// invoking user using one of the supported communications
        /// facilities.
        IoError(facility: IoFacility) {
            description("sudo plugin was unable to perform I/O"),
            display("sudo plugin was unable to perform I/O using facility {}", facility),
        }

        /// An error which can be returned when the requsested plugin API
        /// version is incompatible with the version implemented by this
        /// library.
        UnsupportedApiVersion(cur: Version) {
            description("sudo doesn't support the minimum plugin API version required by this plugin"),
            display("sudo called this plugin with an API version of {}, but a minimum of {} is required", cur, Version::minimum()),
        }

        /// An error which can be returned when there's a general error
        /// when initiailizing the plugin.
        Uninitialized {
            description("the plugin failed to initialize"),
            display("the plugin failed to initialize"),
        }

        /// An error which can be returned if the user is not authorized
        /// to invoke sudo with the provided command and/or options.
        Unauthorized(reason: String) {
            description("command unauthorized"),
            display("command unauthorized: {}", reason),
        }
    }
}

/// A trait that is implemented by all Error types in this library, which
/// allows any error to be converted to its corresponding integer error
/// code as understood by the sudo plugin API.
///
/// The sudo plugin API understands the following error codes:
///
/// *  1: Success
/// *  0: Failure
/// * -1: General error
/// * -2: Usage error
pub trait AsSudoPluginRetval {
    /// Converts the error to its corresponding integer error code.
    fn as_sudo_plugin_retval(&self) -> c_int;
}

impl<T, E: AsSudoPluginRetval> AsSudoPluginRetval for ::std::result::Result<T, E> {
    fn as_sudo_plugin_retval(&self) -> c_int {
        match *self {
            Ok(_)      => 1,
            Err(ref e) => e.as_sudo_plugin_retval(),
        }
    }
}

impl AsSudoPluginRetval for Error {
    fn as_sudo_plugin_retval(&self) -> c_int {
        match *self {
            Error(ErrorKind::Unauthorized(_), _) =>  0,
            Error(_, _)                          => -1,
        }
    }
}
