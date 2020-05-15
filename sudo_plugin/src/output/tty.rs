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

use std::convert::TryFrom;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

///
/// A facility implementing `std::io::Write` that allows printing
/// output to directly to the terminal of the user invoking `sudo`.
///
#[derive(Debug)]
pub struct Tty(File);

impl TryFrom<&Path> for Tty {
    type Error = io::Error;

    fn try_from(path: &Path) -> io::Result<Self> {
        OpenOptions::new().write(true).open(path).map(Tty)
    }
}

impl Write for Tty {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}
