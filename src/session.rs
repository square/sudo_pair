use super::socket::Socket;

use std::collections::HashSet;
use std::io::{Read, Write, Error};
use std::path::Path;
use std::result;

use libc::{uid_t, gid_t, mode_t};

type Result<T> = result::Result<T, Error>;

pub struct Session {
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
    ) -> Result<Session> {
        let mut session = Session {
            socket:  None,
            uid:     uid,
            gids:    gids,
            options: options,
        };

        if !session.is_exempt() {
            session.socket = Some(Socket::open(
                path,
                session.options.socket_uid,
                session.options.socket_gid,
                session.options.socket_mode,
            )?);
        }

        return Ok(session);
    }

    pub fn is_exempt(&self) -> bool {
        // root never requires a pair to deescalate privileges
        if self.is_root() {
            return true
        }

        if self.options.exempt {
            return true
        }

        // exempt if none of our gids are in the set of enforced gids
        if self.gids.is_disjoint(&self.options.gids_enforced) {
            return true
        }

        // exempt if any of our gids are in the set of exempted gids
        if !self.gids.is_disjoint(&self.options.gids_exempted) {
            return true
        }

        false
    }

    pub fn is_root(&self) -> bool {
        self.uid == 0
    }
}

impl Read for Session {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.socket {
            Some(ref mut sock) => sock.read(buf),
            None               => Ok(0),
        }
    }
}

impl Write for Session {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
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
