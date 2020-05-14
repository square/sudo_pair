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

use super::IoEnv;
use crate::errors::Result;

pub trait IoPlugin: Sized {
    const NAME:    &'static str;
    const VERSION: &'static str;

    /// The `sudo_plugin` facility sets `iolog_{facility}` hints that I
    /// believe come from whether or not `LOG_INPUT` or `LOG_OUTPUT` are
    /// set. By default, plugins will not have their logging callbacks
    /// invoked if `sudo` has told us not to log.
    ///
    /// Setting this to `true` will ignore these hints and always call
    /// user-provided log_{std|tty}{in|out|err} callbacks.
    const IGNORE_IOLOG_HINTS : bool = false;

    fn open(env: &'static IoEnv) -> Result<Self>;
    fn close(self, _exit_status: i32, _error: i32) { }

    fn log_ttyin (&mut self, _log: &[u8]) -> Result<()> { Ok(()) }
    fn log_ttyout(&mut self, _log: &[u8]) -> Result<()> { Ok(()) }
    fn log_stdin (&mut self, _log: &[u8]) -> Result<()> { Ok(()) }
    fn log_stdout(&mut self, _log: &[u8]) -> Result<()> { Ok(()) }
    fn log_stderr(&mut self, _log: &[u8]) -> Result<()> { Ok(()) }

    // TODO: support for `change_winsize`
}
