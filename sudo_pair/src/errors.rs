use std::fmt::Display;
use std::result::Result as StdResult;

use failure::{Context, Fail};

pub(crate) use sudo_plugin::errors::{
    Error     as SudoPluginError,
    ErrorKind as SudoPluginErrorKind,
};

pub(crate) type Result<T> = StdResult<T, Error>;

#[derive(Debug, Fail)]
pub(crate) struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub(crate) enum ErrorKind {
    #[fail(display = "couldn't establish communications with the pair")]
    CommunicationError,

    #[fail(display = "pair declined the session")]
    SessionDeclined,

    #[fail(display = "pair ended the session")]
    SessionTerminated,

    #[fail(display = "redirection of stdin to paired sessions is prohibited")]
    StdinRedirected,

    #[fail(display = "the -u and -g options may not both be specified")]
    SudoToUserAndGroup,
}

impl Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { inner: Context::new(kind) }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner: inner }
    }
}

impl From<Error> for SudoPluginError {
    fn from(error: Error) -> SudoPluginError {
        SudoPluginError::with_chain(
            error.compat(),
            SudoPluginErrorKind::Unauthorized
        )
    }
}

impl From<ErrorKind> for SudoPluginError {
    fn from(kind: ErrorKind) -> SudoPluginError {
        SudoPluginError::from(Error::from(kind))
    }
}
