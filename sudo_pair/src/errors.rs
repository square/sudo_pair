use std::fmt::{Display, Formatter, Result as FmtResult};
use std::result::Result as StdResult;

use failure::{Context, Fail};

use sudo_plugin::errors::{
    Error     as SudoPluginError,
    ErrorKind as SudoPluginErrorKind,
};

pub(crate) type Result<T> = StdResult<T, Error>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum ErrorKind {
    CommunicationError,
    SessionDeclined,
    SessionTerminated,
    StdinRedirected,
    SudoToUserAndGroup,
}

impl ErrorKind {
    fn as_str(self) -> &'static str {
        match self {
            ErrorKind::CommunicationError => "couldn't establish communications with the pair",
            ErrorKind::SessionDeclined    => "pair declined the session",
            ErrorKind::SessionTerminated  => "pair ended the session",
            ErrorKind::StdinRedirected    => "redirection of stdin to paired sessions is prohibited",
            ErrorKind::SudoToUserAndGroup => "the -u and -g options may not both be specified",
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.as_str().fmt(f)
    }
}

#[derive(Debug)]
pub(crate) struct Error {
    inner: Context<ErrorKind>,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.inner.fmt(f)
    }
}

impl Fail for Error {}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self::from(Context::new(kind))
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Self {
        Self { inner }
    }
}

///
/// Implements conversion from `Error` to `sudo_plugin::errors::Error`.
/// Since this plugin is security-sensitive, all errors should be
/// converted to an Unauthorized error.
///
impl From<Error> for SudoPluginError {
    fn from(error: Error) -> Self {
        Self::with_chain(
            error.compat(),
            SudoPluginErrorKind::Unauthorized
        )
    }
}

///
/// Also allow converting directly from an `ErrorKind`, which will be
/// implicitly wrapped in a new `Error`.
///
impl From<ErrorKind> for SudoPluginError {
    fn from(kind: ErrorKind) -> Self {
        Self::from(Error::from(kind))
    }
}
