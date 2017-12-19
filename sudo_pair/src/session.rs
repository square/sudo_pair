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

use super::socket::Socket;

use std::collections::HashSet;
use std::io::{Read, Write, Error};
use std::path::{Path, PathBuf};
use std::result;

use libc::{uid_t, gid_t, mode_t};

type Result<T> = result::Result<T, Error>;

pub struct Session {
    path:    PathBuf,
    socket:  Option<Socket>,
    uid:     uid_t,
    gids:    HashSet<gid_t>,

    options: Options,
}

pub struct Options {
    pub socket_uid:  uid_t,
    pub socket_gid:  gid_t,
    pub socket_mode: mode_t,

    pub gids_enforced: HashSet<gid_t>,
    pub gids_exempted: HashSet<gid_t>,

    pub exempt: bool,
}

impl Session {
    pub fn new<P: AsRef<Path>>(
        path:    P,
        uid:     uid_t,
        gids:    HashSet<gid_t>,
        options: Options,
    ) -> Self {
        Self {
            path:    path.as_ref().to_path_buf(),
            socket:  None,
            uid:     uid,
            gids:    gids,
            options: options,
        }
    }

    pub fn is_exempt(&self) -> bool {
        // root never requires a pair to deescalate privileges
        if self.is_root() {
            return true;
        }

        if self.options.exempt {
            return true;
        }

        // exempt if none of our gids are in the set of enforced gids
        if self.gids.is_disjoint(&self.options.gids_enforced) {
            return true;
        }

        // exempt if any of our gids are in the set of exempted gids
        if !self.gids.is_disjoint(&self.options.gids_exempted) {
            return true;
        }

        false
    }

    pub fn is_root(&self) -> bool {
        self.uid == 0
    }

    pub fn close(&mut self) -> Result<()> {
        self.socket.as_mut().map_or(Ok(()), |s| s.close())
    }

    fn connect(&mut self) -> Result<()> {
        if self.socket.is_some() {
            return Ok(());
        }

        if self.is_exempt() {
            return Ok(());
        }

        self.socket = Some(Socket::open(
            &self.path,
            self.options.socket_uid,
            self.options.socket_gid,
            self.options.socket_mode,
        )?);

        Ok(())
    }
}

impl Read for Session {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.connect()?;

        match self.socket {
            Some(ref mut sock) => sock.read(buf),
            None               => Ok(0),
        }
    }
}

impl Write for Session {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.connect()?;

        // if we have a socket, write to it and return the result; if
        // not, pretend we did successfully
        match self.socket {
            Some(ref mut sock) => sock.write(buf),
            None               => Ok(buf.len()),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self.socket {
            Some(ref mut sock) => sock.flush(),
            None               => Ok(()),
        }
    }
}
