//! sudo IO-plugin to require a live human pair.
//!
//! TODO: explain

#![deny(warnings)]

#![warn(anonymous_parameters)]
#![warn(box_pointers)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unstable_features)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]

// this entire crate is practically unsafe code
#![allow(unsafe_code)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(clippy))]
#![cfg_attr(feature="clippy", warn(clippy_pedantic))]

// TODO: disable
#![cfg_attr(feature="clippy", allow(missing_docs_in_private_items))]

extern crate libc;
extern crate unix_socket;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate sudo_plugin;

mod session;
mod socket;

use sudo_plugin::{Result, ResultExt, ErrorKind};
use session::{Session, Options};

use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::str;

use libc::{c_char, c_int, c_uint, sighandler_t, mode_t, pid_t, uid_t, gid_t};

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
        let runas_user = plugin.setting("runas_user").unwrap_or("root");
        let user       = plugin.user_info("user")?;
        let pid        = plugin.user_info("pid")?.parse::<pid_t>().chain_err(|| "user_info['pid'] wasn't a pid" )?;
        let uid        = plugin.user_info("uid")?.parse::<uid_t>().chain_err(|| "user_info['uid'] wasn't a uid" )?;
        let cwd        = plugin.user_info("cwd")?;
        let host       = plugin.user_info("host")?;
        let command    = plugin.command_info("command")?;
        let runas_uid  = plugin.command_info("runas_uid")?.parse::<uid_t>().chain_err(|| "command_info['runas_uid'] wasn't a uid" )?;
        let runas_gid  = plugin.command_info("runas_gid")?.parse::<gid_t>().chain_err(|| "command_info['runas_gid'] wasn't a gid" )?;

        let gids = plugin.command_info("runas_groups")?
            .split(',')
            .filter_map(|gid| gid.parse().ok())
            .collect();

        let options = PluginOptions::from(&plugin.plugin_options);

        // force the session to be exempt if we're running the approval
        // command
        let exempt = options.binary_path == PathBuf::from(&command);

        // encode the original uid into the socket name
        let sockfile = format!("{}.{}.sock", uid, pid);

        let session = Session::new(
            options.socket_dir.join(sockfile),
            uid,
            gids,
            Options {
                socket_uid:    options.socket_uid.unwrap_or(runas_uid),
                socket_gid:    options.socket_gid.unwrap_or(runas_gid),
                socket_mode:   options.socket_mode,
                gids_enforced: options.gids_enforced,
                gids_exempted: options.gids_exempted,
                exempt:        exempt,
            },
        );

        let mut pair = Self {
            session: session,
        };

        if pair.session.is_exempt() {
            return Ok(pair);
        }

        let _ = plugin.print_info(format!(
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
        let sigint = unsafe { signal(libc::SIGINT, ctrl_c as _).expect("Failed to install SIGINT handler") };

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
        pair.session.read_exact(&mut response)?;

        // echo back out the response, since it's noecho, raw on the client
        let _ = pair.session.write_all(&response[..]);
        let _ = pair.session.write_all(b"\n");

        // restore the original SIGINT handler
        // TODO: handle errors
        let _ = unsafe { signal(libc::SIGINT, sigint).expect("Failed to install SIGINT handler") };

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
        self.session.write_all(log).chain_err(|| ErrorKind::Unauthorized("pair abandoned session".to_string()))
    }

    fn log_disabled(&mut self, _: &[u8]) -> Result<()> {
        if self.session.is_exempt() {
            return Ok(());
        }

        bail!(ErrorKind::Unauthorized("redirection of stdin, stout, and stderr prohibited".to_string()))
    }
}

fn parse_delimited_string<F, T>(
    string: &str,
    delimiter: char,
    parser: F,
) -> HashSet<T>
    where F: FnMut(&str) -> T, T: Eq + std::hash::Hash {
    string.split(delimiter).map(parser).collect()
}

unsafe fn signal(signum: c_int, handler: sighandler_t) -> io::Result<sighandler_t> {
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

struct PluginOptions {
    binary_path: PathBuf,
    socket_dir:  PathBuf,
    socket_uid:  Option<uid_t>,
    socket_gid:  Option<gid_t>,
    socket_mode: mode_t,

    gids_enforced: HashSet<gid_t>,
    gids_exempted: HashSet<gid_t>,
}

impl Default for PluginOptions {
    fn default() -> Self {
        Self {
            binary_path:   PathBuf::from("/usr/bin/sudo_pair_approve"),
            socket_dir:    PathBuf::from("/var/run/sudo_pair"),
            socket_uid:    None,
            socket_gid:    None,
            socket_mode:   0o700,
            gids_enforced: HashSet::new(),
            gids_exempted: HashSet::new(),
        }
    }
}

impl<'a> From<&'a HashMap<String, String>> for PluginOptions {
    fn from(map: &'a HashMap<String, String>) -> Self {
        let mut options = Self::default();

        for (key, value) in map {
            match &key[..] {
                "BinaryPath"   => options.binary_path   = PathBuf::from(value),
                "SocketDir"    => options.socket_dir    = PathBuf::from(value),
                "SocketUid"    => options.socket_uid    = Some(value.parse().expect("SocketUid must be an integer")),
                "SocketGid"    => options.socket_gid    = Some(value.parse().expect("SocketGid must be an integer")),
                "SocketMode"   => options.socket_mode   = mode_t::from_str_radix(value, 8).expect("SocketMode must be a base-8 integer"),
                "GidsEnforced" => options.gids_enforced = parse_delimited_string(value, ',', |s| s.parse().expect("GidsEnforced must be a comma-separated list of integers")),
                "GidsExempted" => options.gids_exempted = parse_delimited_string(value, ',', |s| s.parse().expect("GidsExempted must be a comma-separated list of integers")),
                _              => (), // TODO: warn
            }
        }

        options
    }
}
