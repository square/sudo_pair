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

use crate::errors::Error;
use crate::output::Level;

use sudo_plugin_sys::sudo_printf_t;

use std::ffi::CString;
use std::io::{self, Write};

/// A facility implementing `std::io::Write` that allows printing
/// output to the user invoking `sudo`. Technically, the user may
/// not be present on a local tty, but this will be wired up to a
/// `printf`-like function that outputs to either STDOUT or STDERR.
#[derive(Clone, Debug)]
pub struct PrintFacility {
    /// A function pointer to an underlying printf facility provided
    /// by the sudo_plugin API.
    facility: sudo_printf_t,

    /// The [`Level`] to send messages at. The sudo_plugin API only
    /// supports distinguishing between informational messages and error
    /// messages.
    level: Level,

    /// An optional tag to prepend to any logged messages.
    tag: Vec<u8>,
}

impl PrintFacility {
    /// Constructs a new `PrintFacility` that emits output to the user invoking
    /// `sudo`. A tuple is returned that presents a handle to write to `stdout`
    /// and `stderr` in that order.
    ///
    /// An optional `name` can be provided. If it is, the [`write_line`] and
    /// [`write_error`] methods will emit the name as a prefix for each line.
    ///
    /// # Safety
    ///
    /// This function *must* be provided with either a `None` or a real pointer
    /// to a `printf`-style function. Once provided to this function, the
    /// function pointer should be discarded at never used, as it is unsafe for
    /// this function to be called concurrently.
    #[must_use]
    pub unsafe fn new(name: Option<&str>, printf: sudo_printf_t) -> (Self, Self) {
        let tag: Vec<u8> = name
            .map(|name| format!("{}: ", name).into())
            .unwrap_or_default();

        let     stdout = Self { tag, facility: printf, level: Level::Info };
        let mut stderr = stdout.clone();

        stderr.level = Level::Error;

        (stdout, stderr)
    }

    /// Pretty-prints a line, prefixed by the name of the plugin.
    pub fn write_line(&mut self, line: &[u8]) -> io::Result<()> {
        let tag = self.tag.clone();

        self.write_all(tag.as_slice())?;
        self.write_all(line)?;
        self.write_all(b"\n")?;

        Ok(())
    }

    /// Pretty-prints nested errors to the user.
    pub fn write_error(&mut self, error: &Error) -> io::Result<()> {
        // errors are prefixed with a newline for clarity, since they
        // might be emitted while an existing line has output on it
        self.write_all(b"\n")?;

        for e in error.iter() {
            self.write_line(format!("{}", e).as_bytes())?;
        }

        Ok(())
    }
}

impl Write for PrintFacility {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let printf = self.facility.ok_or_else(||
            io::Error::new(io::ErrorKind::NotConnected, "no printf provided")
        )?;

        let message = CString::new(buf).map_err(|err|
            io::Error::new(io::ErrorKind::InvalidData, err)
        )?;

        let count = unsafe {
            // TODO: level should be bitflags when we start implementing the
            // full conversation interface
            (printf)(self.level as _, message.as_ptr())
        };

        #[allow(clippy::cast_sign_loss)]
        match count {
            c if c < 0 => Err(io::Error::last_os_error()),
            c          => Ok(c as _)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
