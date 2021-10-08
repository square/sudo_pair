// Copyright 2020 Square Inc.
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

// TODO: once associated type defaults are stabilized, provide defaults
// for `Error`, and use that to return `Disable` for default methods in
// order to avoid any performance penalties from calling unimplemented
// functions.

use super::IoEnv;
use crate::errors::SudoError;

#[warn(clippy::missing_inline_in_public_items)]

/// The trait that defines the implementation of a sudo I/O plugin.
pub trait IoPlugin: 'static + Sized {
    /// The type for errors returned by this `IoPlugin`. Errors must
    /// implement the [`SudoError`](crate::errors::SudoError) trait
    /// which describes how to convert them to the return codes expected
    /// by the `sudo_plugin(8)` facility.
    type Error: SudoError;

    /// The name of the plugin. Used when printing the version of the
    /// plugin and error messages.
    const NAME: &'static str;

    /// The version of the plugin. Defaults to the the value of the
    /// `CARGO_PKG_VERSION` environment variable during build.
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    /// The `sudo_plugin` facility sets `iolog_{facility}` hints that I
    /// believe come from whether or not `LOG_INPUT` or `LOG_OUTPUT` are
    /// set. By default, plugins will not have their logging callbacks
    /// invoked if `sudo` has told us not to log.
    ///
    /// Setting this to `true` will ignore these hints and always call
    /// user-provided `log_*` callbacks.
    const IGNORE_IOLOG_HINTS: bool = false;

    /// Prints the name and version of the plugin. A default
    /// implementation of this function is provided, but may be
    /// overridden if desired.
    #[inline]
    fn show_version(env: &IoEnv, _verbose: bool) {
        use std::io::Write;

        let _ = writeln!(
            env.stdout(),
            "{} I/O plugin version {}",
            Self::NAME,
            Self::VERSION,
        );
    }

    /// The `open` function is run before the `log_ttyin`, `log_ttyout`,
    /// `log_stdin`, `log_stdout`, `log_stderr`, `log_suspend`, or
    /// `change_winsize` methods are called. It is only called if the
    /// policy plugin's `check_policy` function has returned
    /// successfully.
    ///
    /// # Errors
    ///
    /// Any errors will be recursively printed (up their
    /// [`source`](std::error::Error::source) chain) then converted to
    /// an [`OpenStatus`](crate::errors::OpenStatus) before being
    /// returned to `sudo`.
    fn open(env: &'static IoEnv) -> Result<Self, Self::Error>;

    /// The `close` method is called when the command being run by
    /// sudo finishes. A default no-op implementation is provided, but
    /// be overriden if desired.
    ///
    /// As suggested by its signature, once this method exits, the
    /// plugin will be dropped.
    #[inline]
    fn close(self, _exit_status: i32, _error: i32) {}

    /// The `log_ttyin` method is called whenever data can be read from the user
    /// but before it is passed to the running command. This allows the plugin
    /// to reject data if it chooses to (for instance if the input contains
    /// banned content).
    ///
    /// # Errors
    ///
    /// Any errors will be recursively printed (up their
    /// [`source`](std::error::Error::source) chain) then converted to
    /// a [`LogStatus`](crate::errors::LogStatus) before being
    /// returned to `sudo`.
    #[inline]
    fn log_ttyin(&self, _log: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    /// The `log_ttyout` function is called whenever data can be read from the
    /// command but before it is written to the user's terminal. This allows the
    /// plugin to reject data if it chooses to (for instance if the output
    /// contains banned content).
    ///
    /// # Errors
    ///
    /// Any errors will be recursively printed (up their
    /// [`source`](std::error::Error::source) chain) then converted to
    /// a [`LogStatus`](crate::errors::LogStatus) before being
    /// returned to `sudo`.
    #[inline]
    fn log_ttyout(&self, _log: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    /// The `log_stdin` function is only used if the standard input does not
    /// correspond to a tty device. It is called whenever data can be read from
    /// the standard input but before it is passed to the running command.
    ///
    /// # Errors
    ///
    /// Any errors will be recursively printed (up their
    /// [`source`](std::error::Error::source) chain) then converted to
    /// a [`LogStatus`](crate::errors::LogStatus) before being
    /// returned to `sudo`.
    #[inline]
    fn log_stdin(&self, _log: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    /// The `log_stdout` function is only used if the standard output does not
    /// correspond to a tty device. It is called whenever data can be read from
    /// the command but before it is written to the standard output. This allows
    /// the plugin to reject data if it chooses to (for instance if the output
    /// contains banned content).
    ///
    /// # Errors
    ///
    /// Any errors will be recursively printed (up their
    /// [`source`](std::error::Error::source) chain) then converted to
    /// a [`LogStatus`](crate::errors::LogStatus) before being
    /// returned to `sudo`.
    #[inline]
    fn log_stdout(&self, _log: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    /// The `log_stderr` function is only used if the standard error does not
    /// correspond to a tty device. It is called whenever data can be read from
    /// the command but before it is written to the standard error. This allows
    /// the plugin to reject data if it chooses to (for instance if the output
    /// contains banned content).
    ///
    /// # Errors
    ///
    /// Any errors will be recursively printed (up their
    /// [`source`](std::error::Error::source) chain) then converted to
    /// a [`LogStatus`](crate::errors::LogStatus) before being
    /// returned to `sudo`.
    #[inline]
    fn log_stderr(&self, _log: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    /// The `change_winsize` callback is invoked whenever the controlling
    /// terminal for the sudo session detects that it's been resized. It is
    /// provided with a count of the number of horizontal lines and vertical
    /// columns that the terminal can now display.
    ///
    /// # Errors
    ///
    /// Any errors will be recursively printed (up their
    /// [`source`](std::error::Error::source) chain) then converted to
    /// a [`LogStatus`](crate::errors::LogStatus) before being
    /// returned to `sudo`.
    #[inline]
    fn change_winsize(&self, _lines: u64, _cols: u64) -> Result<(), Self::Error> {
        Ok(())
    }

    // TODO: support for `log_suspend`
}
