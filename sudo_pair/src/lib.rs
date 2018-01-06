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

//! sudo IO-plugin to require a live human pair.
//!
//! TODO: explain

// TODO: remove all unwraps
// TODO: remove all to_string_lossy
// TODO: switch from error_chain to failure crate?

#![deny(warnings)]

#![warn(anonymous_parameters)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unstable_features)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", warn(clippy))]
#![cfg_attr(feature = "clippy", warn(clippy_pedantic))]

// this library is fundamentally built upon unsafe code
#![allow(unsafe_code)]

extern crate libc;
extern crate unix_socket;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate sudo_plugin;

mod session;
mod socket;

use session::{Options, Session};

use std::collections::HashSet;
use std::io::{self, Read, Write};
use std::iter::FromIterator;
use std::path::PathBuf;

use libc::{c_char, c_int, c_uint, gid_t, mode_t, sighandler_t, uid_t};

use sudo_plugin::OptionMap;

const DEFAULT_BINARY_PATH : &'static str = "/usr/bin/sudo_pair_approve";
const DEFAULT_SOCKET_DIR  : &'static str = "/var/run/sudo_pair";

sudo_io_plugin! {
     sudo_pair: SudoPair {
        close:      close,
        log_ttyout: log_ttyout,
        log_stdin:  log_disabled,
        log_stdout: log_disabled,
        log_stderr: log_disabled,
     }
}

struct SudoPair {
    session: Session,
}

impl SudoPair {
    fn open(plugin: &'static sudo_plugin::Plugin) -> Result<Self> {
        // if `runas_user` wasn't provided (via the `-u` flag), it means
        // we're sudoing to root
        let runas_user = plugin.settings.runas_user.as_ref().map_or("root", String::as_str);
        let user       = &plugin.user_info.user;
        let cwd        = &plugin.user_info.cwd;
        let host       = &plugin.user_info.host;
        let pid        = &plugin.user_info.pid;
        let uid        = &plugin.user_info.uid;
        let command    = &plugin.command_info.command;
        let runas_uid  = &plugin.command_info.runas_uid;
        let runas_gid  = &plugin.command_info.runas_gid;

        let gids: HashSet<gid_t> = HashSet::from_iter(
            plugin.command_info.runas_groups.iter().cloned(),
        );

        let options = PluginOptions::from(&plugin.plugin_options);

        // force the session to be exempt if we're running the approval
        // command
        let exempt = options.binary_path == PathBuf::from(&command);

        // encode the original uid into the socket name
        let sockfile = format!("{}.{}.sock", *uid, *pid);

        let session = Session::new(
            options.socket_dir.join(sockfile),
            *uid,
            gids,
            Options {
                socket_uid:    options.socket_uid.unwrap_or(*runas_uid),
                socket_gid:    options.socket_gid.unwrap_or(*runas_gid),
                socket_mode:   options.socket_mode,
                gids_enforced: HashSet::from_iter(options.gids_enforced.iter().cloned()),
                gids_exempted: HashSet::from_iter(options.gids_exempted.iter().cloned()),
                exempt:        exempt,
            },
        );

        let mut pair = Self {
            session: session,
        };

        if pair.session.is_exempt() {
            return Ok(pair);
        }

        let _ = plugin.print_info(&format!(
            "Running this command requires another user to approve and watch \
             your session. Please have another user run\n\
             \n\
             \tsudo_pair_approve {} {} {} {}\n",
            host,
            user,
            runas_user,
            pid,
        ));

        // temporarily install a SIGINT handler while we block on accept()
        // TODO: handle errors
        let sigint = unsafe {
            signal(libc::SIGINT, ctrl_c as _)
                .expect("Failed to install SIGINT handler")
        };

        // TODO: handle return value
        let _ = pair.session.write_all(
            format!("\
    User {} is attempting to run

    \t<\x1b[1;34m{}\x1b[0m@\x1b[1;32m{}\x1b[0m:\x1b[01;34m{}\x1b[0m\x1b[1;32m $\x1b[0m > sudo -u {} {}

    If you approve, you will see the live session through this terminal. To \
    immediately abort the interactive session (and kill the running sudo \
    session), press Ctrl-D (EOF).

    Please note: if you abandon this session, it will kill the running sudo \
    session.

    Approve? y/n [n]: ",
                user,
                user,
                host,
                cwd,
                runas_user,
                command,
            ).as_bytes()
        );

        let mut response = [0];

        // read one byte from the socket
        pair.session
            .read_exact(&mut response)
            .chain_err(|| "failed to read approval from the pair")?;

        // echo back out the response, since it's noecho, raw on the client
        let _ = pair.session.write_all(&response[..]);
        let _ = pair.session.write_all(b"\n");

        // restore the original SIGINT handler
        // TODO: handle errors
        let _ = unsafe {
            signal(libc::SIGINT, sigint)
                .expect("Failed to install SIGINT handler")
        };

        // if those two bytes were a "yes", we're authorized to
        // open a session; otherwise we've been declined
        match &response {
            b"y" | b"Y" => Ok(pair),
            _           => bail!(ErrorKind::Unauthorized("denied by pair".to_string())),
        }
    }

    fn close(&mut self, _: i32, _: i32) {
        let _ = self.session.close();
    }

    fn log_ttyout(&mut self, log: &[u8]) -> Result<()> {
        self.session.write_all(log).chain_err(|| {
            ErrorKind::Unauthorized("pair abandoned session".to_string())
        })
    }

    fn log_disabled(&mut self, _: &[u8]) -> Result<()> {
        if self.session.is_exempt() {
            return Ok(());
        }

        bail!(ErrorKind::Unauthorized(
            "redirection of stdin, stout, and stderr prohibited".to_string()
        ))
    }
}

unsafe fn signal(
    signum:  c_int,
    handler: sighandler_t,
) -> io::Result<sighandler_t> {
    match libc::signal(signum, handler) {
        libc::SIG_ERR => Err(io::Error::last_os_error()),
        previous      => Ok(previous),
    }
}

// TODO: There's not much we can do in a signal handler, but
// _exit(3) is safe (exit(3) isn't!). Ideally we'd close(2) the
// socket blocking on accept(2) instead of exiting directly, but
// that's more complicated than it's worth for now.
unsafe extern "C" fn ctrl_c(_sig: c_int) {
    // sudo normally exits with exit code 1 if you Ctrl-C during
    // password entry, so we retain that convention
    libc::_exit(1);
}

#[derive(Debug)]
struct PluginOptions {
    // `BinaryPath` is the location of the approval binary, so that we
    // can bypass the approval process for invoking it
    //
    // Default: `"/usr/bin/sudo_pair_approve"`
    binary_path: PathBuf,

    // `SocketDir` is the path where this plugin will store sockets for
    // sessions that are pending approval
    //
    // Default: `"/var/run/sudo_pair"`
    socket_dir: PathBuf,

    // `SocketUid` is the owner uid for sockets for sessions that are
    // pending approval. If `None`, will be set to the uid of the user
    // being sudoed to. This allows a use-case where approval can only
    // be granted by someone also authorized to sudo to the target user.
    //
    // Default: `None`
    socket_uid: Option<uid_t>,

    // `SocketGid` is the owner gid for sockets for sessions that are
    // pending approval. If `None`, will be set to the gid of the user
    // being sudoed to. This allows a use-case where approval can only
    // be granted by someone also authorized to sudo to the target
    // users' group.
    //
    // Default: `None`
    socket_gid: Option<gid_t>,

    // `SocketMode` is the mode permissions for the socket on disk.
    // Changing this to be user-writable or group-writable allows anyone
    // in the target user or target user's group to approve the session,
    // respectively.
    //
    // Default: `0o700`
    socket_mode: mode_t,

    // TODO: doc
    gids_enforced: Vec<gid_t>,
    gids_exempted: Vec<gid_t>,
}

impl<'a> From<&'a OptionMap> for PluginOptions {
    fn from(map: &'a OptionMap) -> Self {
        Self {
            binary_path:   map.get_parsed("BinaryPath")  .unwrap_or(DEFAULT_BINARY_PATH.into()),
            socket_dir:    map.get_parsed("SocketDir")   .unwrap_or(DEFAULT_SOCKET_DIR.into()),
            socket_uid:    map.get_parsed("SocketUid")   .ok(),
            socket_gid:    map.get_parsed("SocketGid")   .ok(),
            socket_mode:   map.get_parsed("SocketMode")  .unwrap_or(0o700),
            gids_enforced: map.get_parsed("GidsEnforced").unwrap_or(vec![]),
            gids_exempted: map.get_parsed("GidsExempted").unwrap_or(vec![]),
        }
    }
}
