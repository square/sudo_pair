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
//! This plugin implements dual control for `sudo`, requiring that
//! another engineer approve and monitor any privileged sessions.

// TODO: remove all to_string_lossy
// TODO: error message when /var/run/sudo_pair missing
// TODO: iolog in `sudoreplay(8)` format
// TODO: rustfmt
// TODO: double-check all `as`-casts
// TODO: docs on docs.rs
// TODO: various badges
// TODO: fill out all fields of https://doc.rust-lang.org/cargo/reference/manifest.html
// TODO: implement change_winsize

#![warn(bad_style)]
#![warn(future_incompatible)]
#![warn(nonstandard_style)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(rustdoc)]
#![warn(unused)]

#![warn(bare_trait_objects)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unstable_features)]
#![warn(unused_import_braces)]
#![warn(unused_lifetimes)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]

// this entire crate is unsafe code
#![allow(unsafe_code)]

#![cfg_attr(feature="cargo-clippy", warn(clippy::all))]

mod errors;
mod template;
mod socket;

use crate::errors::*;
use crate::template::Spec;
use crate::socket::Socket;

use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use libc::{gid_t, mode_t, uid_t};

use failure::ResultExt;

use sudo_plugin::{sudo_io_plugin, IoEnv, IoPlugin, options::OptionMap};

const DEFAULT_BINARY_PATH      : &str       = "/usr/bin/sudo_approve";
const DEFAULT_USER_PROMPT_PATH : &str       = "/etc/sudo_pair.prompt.user";
const DEFAULT_PAIR_PROMPT_PATH : &str       = "/etc/sudo_pair.prompt.pair";
const DEFAULT_SOCKET_DIR       : &str       = "/var/run/sudo_pair";
const DEFAULT_GIDS_ENFORCED    : [gid_t; 1] = [0];

const DEFAULT_USER_PROMPT : &[u8] = b"%B '%p %u'\n";
const DEFAULT_PAIR_PROMPT : &[u8] = b"%U@%h:%d$ %C\ny/n? [n]: ";

sudo_io_plugin!{ sudo_pair : SudoPair }

struct SudoPair {
    env:     &'static IoEnv,
    options: PluginOptions,
    socket:  Option<Socket>,

    slog: slog::Logger,
}

impl IoPlugin for SudoPair {
    const NAME:     &'static str = "sudo_pair";
    const VERSION : &'static str = env!("CARGO_PKG_VERSION");

    fn open(env: &'static IoEnv) -> sudo_plugin::errors::Result<Self> {
        let mut slog = slog(Self::NAME, Self::VERSION);

        slog::debug!(slog, "plugin initializing");

        let args : Vec<_> = env.cmdline.iter()
            .skip(1)
            .map (|arg| arg.to_string_lossy())
            .collect();

        slog = slog::Logger::new(&slog, slog::o!(
            "uid"           => &env.user_info.uid,
            "runas_euid"    => &env.command_info.runas_euid,
            "runas_egid"    => &env.command_info.runas_egid,
            "command"       => env.command_info.command.to_string_lossy().into_owned(),
            "args"          => format!("{:?}", args),
        ));

        let options = PluginOptions::from(&env.plugin_options);

        slog::debug!(slog, "initialized with plugin options:";
             "plugin_options" => &options
        );

        // TODO: convert all outgoing errors to be unauthorized errors
        let mut pair = Self {
            env,
            options,
            socket:  None,

            slog,
        };

        if pair.is_exempt() {
            slog::info!(pair.slog, "pair session exempt from pairing requirements");

            return Ok(pair)
        }

        slog::info!(pair.slog, "pair session required");

        if pair.is_sudoing_to_user_and_group() {
            slog::error!(pair.slog, "both -u and -g were provided to sudo"; slog::o!(
                "user"  => &pair.env.settings.runas_user,
                "group" => &pair.env.settings.runas_group,
            ));

            return Err(ErrorKind::SudoToUserAndGroup.into());
        }

        let template_spec = pair.template_spec();

        pair.local_pair_prompt(&template_spec);
        pair.remote_pair_connect()?;
        pair.remote_pair_prompt(&template_spec)?;

        // TODO(security): provide a configurable option to deny or log
        // if the remote euid is the same as the local euid. For some
        // reason I convinced myself that this is necessary to implement
        // in the client and not the pair plugin, but I can't remember
        // what the reasoning was at the moment.
        //
        // Oh, now I remember. It *has* to be done on the client,
        // because the approval script is run under `sudo` itself so
        // that we can verify the pairer is also capable of doing the
        // task the user invoking `sudo` is trying to do. Unfortunately,
        // the OS APIs we have to determine the other side of the
        // connection only tell us the *euid*, not the *uid*. So we end
        // up with the euid of `root` which isn't helpful. So this kind
        // of check *must* be done on the client.
        //
        // Except I have an idea for how to solve this plugin-side. Open
        // a socket writable by all. When someone connects, get the
        // credentials of the peer and send them a cryptographically-
        // random token. Close the socket and reopen a new one as we
        // currently do. Instead of expecting a `y`, expect the token.
        // This binds their ability to approve the session (able to
        // write to the socket) with their original identity (proven
        // through providing the token from their original user). This
        // shouldn't be too hard, but I haven't gotten around to it yet.

        slog::info!(pair.slog, "pair session started");

        Ok(pair)
    }

    fn close(mut self, _: i32, _: i32) {
        // if we have a socket, close it
        if let Some(mut socket) = self.socket.take() {
            slog::trace!(self.slog, "pair session ending");
            let _ = socket.close();
            slog::info!(self.slog, "pair session ended");
        }
    }

    fn log_ttyout(&mut self, log: &[u8]) -> sudo_plugin::errors::Result<()> {
        self.log_output(log).map_err(|e| e.into())
    }

    fn log_stdout(&mut self, log: &[u8]) -> sudo_plugin::errors::Result<()> {
       self.log_output(log).map_err(|e| e.into())
    }

    fn log_stderr(&mut self, log: &[u8]) -> sudo_plugin::errors::Result<()> {
        self.log_output(log).map_err(|e| e.into())
    }

    fn log_stdin(&mut self, _: &[u8]) -> sudo_plugin::errors::Result<()> {
        // if we're exempt, don't disable stdin
        if self.is_exempt() {
            return Ok(());
        }

        Err(ErrorKind::StdinRedirected.into())
    }
}

impl SudoPair {
    fn log_output(&mut self, log: &[u8]) -> Result<()> {
        // if we have a socket, write to it
        self.socket.as_mut().map_or(Ok(()), |socket| {
            socket.write_all(log)
        }).context(ErrorKind::SessionTerminated)?;

        slog::trace!(self.slog, "{{{} bytes sent}}", log.len());

        Ok(())
    }

    fn local_pair_prompt(&self, template_spec: &Spec) {
        // read the template from the file; if there's an error, use the
        // default template instead
        let template : Vec<u8> = File::open(&self.options.user_prompt_path)
            .and_then(|file| file.bytes().collect() )
            .unwrap_or_else(|_| DEFAULT_USER_PROMPT.into() );

        slog::trace!(self.slog, "local prompt template loaded");

        let prompt = template_spec.expand(&template[..]);

        // NOTE: I don't think it's adviseable to log the evaluated
        // template here since it likely contains ANSI escape sequences
        // that clear the terminal, adjust color/width, etc.
        slog::trace!(self.slog, "local prompt template evaluated");

        // If sudo has detected the user's TTY, we try to print to it
        // directly. If we don't have a TTY or fail to open/write to
        // it, we fall back to writing with the plugin's printf
        // function. This allows `sudo_pair` to be used in situations
        // where stdout/stderr are redirected to pipes.
        //
        // we ignore any errors about printing the prompt locally,
        // because we can't really do anything productive other than
        // die, and that could render `sudo` inoperable given an
        // unanticipated bug (however, if we fail to print to the TTY)
        // we do try to print directly to stderr
        //
        // TODO: the stderr write is returning an error (EINVAL) even
        // though it prints successfully; I'm not entirely sure why. It
        // started failing when I added some new operators for the
        // templating code, but nothing in that commit seems like it
        // should have obviously started causing writes to fail.
        //
        // EINVAL is raised by the underlying libc vfprintf call, which
        // appears to only be problematic if the underlying write fails.
        // As far as I can tell, this only happens if something isn't
        // aligned correctly and the `fd` is opened with `O_DIRECT`. But
        // it seems unlikely that STDIN is opened that way or that
        // anything Rust allocates is misaligned. The other possibility
        // is that STDIN is "unsuitable for writing" which also seems
        // improbable. For now, I'm ignoring the situation but hopefully
        // there's enough information here for someone (probably me) to
        // pick up where I left off.
        let _ = self.env.tty().as_mut()
            .and_then(|tty| tty.write_all(&prompt).ok() )
            .ok_or_else(|| self.env.stderr().write_all(&prompt));

        slog::trace!(self.slog, "local prompt rendered");
    }

    fn remote_pair_connect(&mut self) -> Result<()> {
        let slog = slog::Logger::new(&self.slog, slog::o!(
            "socket_path" => self.socket_path().to_string_lossy().into_owned(),
        ));

        slog::debug!(slog, "socket initializing";
            "socket_uid"  => self.socket_uid(),
            "socket_gid"  => self.socket_gid(),
            "socket_mode" => format!("{:#06o}", self.socket_mode()),
        );

        if self.socket.is_some() {
            slog::warn!(slog, "socket unexpectedly already initialized");

            // TODO: this is probably an error, since we should never
            // expect the socket to have already been created
            return Ok(());
        }

        slog::info!(slog, "socket waiting for pair to connect...");

        // TODO: clearly indicate when the socket path is missing
        // this is currently being hidden by the `context` method which
        // ironically hides the extra context instead of providing extra
        // context
        let socket = Socket::open(
            self.socket_path(),
            self.socket_uid(),
            self.socket_gid(),
            self.socket_mode(),
        ).context(ErrorKind::CommunicationError)?;

        self.socket = Some(socket);

        slog::info!(slog, "socket connected");

        Ok(())
    }

    fn remote_pair_prompt(&mut self, template_spec: &Spec) -> Result<()> {
        // read the template from the file; if there's an error, use the
        // default template instead
        let template : Vec<u8> = File::open(&self.options.pair_prompt_path)
            .and_then(|file| file.bytes().collect() )
            .unwrap_or_else(|_| DEFAULT_PAIR_PROMPT.into() );

        slog::trace!(self.slog, "remote prompt loaded");

        let prompt = template_spec.expand(&template[..]);

        slog::trace!(self.slog, "remote prompt evaluated");

        let socket = self.socket
            .as_mut()
            .ok_or(ErrorKind::CommunicationError)?;

        socket.write_all(&prompt[..])
            .context(ErrorKind::CommunicationError)?;

        socket.flush()
            .context(ErrorKind::CommunicationError)?;


        slog::trace!(self.slog, "remote prompt rendered");

        // default `response` to something other than success, since
        // `read` might return without actually having written anything;
        // this prevents us from being required to check the number of
        // bytes actually read from `read`
        let mut response : [u8; 1] = [b'n'];

        slog::debug!(self.slog, "remote prompt awaiting response...");

        // read exactly one byte back from the socket for the
        // response (`read_exact` isn't used because it will capture
        // Ctrl-C and retry the read); we don't need to check the return
        // value because if the read was successful, we're guaranteed to
        // have read at least one byte
        let _ = socket.read(&mut response)
            .context(ErrorKind::SessionDeclined)?;

        slog::debug!(self.slog, "remote pair responded";
            "response" => String::from_utf8_lossy(&response[..]).into_owned(),
        );

        // echo back out the response, since the client is anticipated
        // to be noecho
        let _ = socket.write_all(&response[..]);
        let _ = socket.write_all(b"\n");

        match &response {
            b"y" | b"Y" => (),
            _           => {
                slog::warn!(self.slog, "remote pair declined session");
                return Err(ErrorKind::SessionDeclined.into());
            }
        };

        slog::info!(self.slog, "remote pair approved session");

        Ok(())
    }

    fn is_exempt(&self) -> bool {
        // root is always exempt
        if self.is_sudoing_from_root() {
            slog::debug!(self.slog, "sudo initiated by root";
                "user_info.uid" => self.env.user_info.uid,
            );

            return true;
        }

        // a user sudoing entirely to themselves is weird, but I can't
        // see any reason not to let them do it without approval since
        // they can already do everything as themselves anyway
        if self.is_sudoing_to_themselves() {
            slog::debug!(self.slog, "sudo to current user";
                "user_info.uid"          => self.env.user_info.uid,
                "command_info.runas_uid" => self.env.command_info.runas_uid,
            );

            return true;
        }

        // exempt if the approval command is the command being invoked
        if self.is_sudoing_approval_command() {
            slog::debug!(self.slog, "sudo running approval command";
                "command_info.command"       => self.env.command_info.command.to_string_lossy().into_owned(),
                "plugin_options.binary_path" => self.options.binary_path.to_string_lossy().into_owned(),
            );

            return true;
        }

        // policy plugins can inform us that logging is unnecessary
        if self.is_exempted_from_logging() {
            slog::debug!(self.slog, "sudo command exempted from logging");

            return true;
        }

        // exempt if the user who's sudoing is in a group that's exempt
        // from having to pair
        if self.is_sudoing_from_exempted_gid() {
            slog::debug!(self.slog, "sudo from exempt group id");

            return true;
        }

        // exempt if none of the gids of the user we're sudoing into are
        // in the set of gids we enforce pairing for
        if !self.is_sudoing_to_enforced_gid() {
            slog::debug!(self.slog, "sudo to unenforced group id");

            return true;
        }

        slog::debug!(self.slog, "sudo session requires a pair");

        false
    }

    fn is_sudoing_from_root(&self) -> bool {
        // theoretically, root's `uid` should be 0, but it's probably
        // safest to check whatever user `sudo` is running as since sudo
        // is pretty much by definition going to be running setuid;
        // hypothetically with selinux someone could have sudo owned by
        // some non-root user that has the caps needed for sudoing around
        //
        // note that the `euid` will always be the owner of the `sudo`
        // binary
        self.env.user_info.uid == self.env.user_info.euid
    }

    fn is_sudoing_to_themselves(&self) -> bool {
        // if they're not sudoing to a new uid or to a new gid, they're
        // just becoming themselves... right?
        if !self.is_sudoing_to_user() && !self.is_sudoing_to_group() {
            debug_assert_eq!(
                self.env.runas_gids(),
                self.env.user_info.groups.iter().cloned().collect()
            );

            return true;
        }

        false
    }

    fn is_sudoing_approval_command(&self) -> bool {
        self.env.command_info.command == self.options.binary_path
    }

    ///
    /// Returns true if the policy plugin has not given us any
    /// facilities to log output for.
    ///
    fn is_exempted_from_logging(&self) -> bool {
        if
            !self.env.command_info.iolog_ttyout &&
            !self.env.command_info.iolog_stdout &&
            !self.env.command_info.iolog_stderr
        {
            return true;
        }

        false
    }

    fn is_sudoing_to_user_and_group(&self) -> bool {
        // if a user is doing `sudo -u ${u} -g ${g}`, we don't have a
        // way to ensure that the pair can act with permissions of both
        // the new user and the new group; ignoring this would allow
        // someone to gain a group privilege through a pair who doesn't
        // also have that group privilege
        //
        // note that we don't use `is_sudoing_to_group` because sudoing
        // to a new user typically implicitly comes along with sudoing
        // to a new group which is fine, what we want to avoid is the
        // user explicitly providing a *different* group
        if self.is_sudoing_to_user() && self.is_sudoing_to_explicit_group() {
            return true
        }

        false
    }

    fn is_sudoing_from_exempted_gid(&self) -> bool {
        !self.options.gids_exempted.is_disjoint(
            &self.env.user_info.groups.iter().cloned().collect()
        )
    }

    fn is_sudoing_to_enforced_gid(&self) -> bool {
        !self.options.gids_enforced.is_disjoint(
            &self.env.runas_gids()
        )
    }

    fn is_sudoing_to_user(&self) -> bool {
        // `plugin.settings.runas_user` tells us the value of `-u`, but
        // by checking the change in uid, we can exclude cases where
        // they're sudoing to themselves
        self.env.user_info.uid != self.env.command_info.runas_euid
    }

    fn is_sudoing_to_group(&self) -> bool {
        self.env.user_info.gid != self.env.command_info.runas_egid
    }

    // returns true if `-g` was specified
    fn is_sudoing_to_explicit_group(&self) -> bool {
        self.env.settings.runas_group.is_some()
    }

    fn socket_path(&self) -> PathBuf {
        // we encode the originating `uid` into the pathname since
        // there's no other (easy) way for the approval command to probe
        // for this information
        //
        // note that we want the *`uid`* and not the `euid` here since
        // we want to know who the real user is and not the `uid` of the
        // owner of `sudo`
        self.options.socket_dir.join(
            format!(
                "{}.{}.sock",
                self.env.user_info.uid,
                self.env.user_info.pid,
            )
        )
    }

    fn socket_uid(&self) -> uid_t {
        // we explicitly want to have the socket owned by the root user
        // if we're doing `sudo -g`, so that the sudoing user can't
        // silently self-approve by manually connecting to the socket
        // without needing to invoke sudo
        if self.is_sudoing_to_user() {
            self.env.command_info.runas_euid
        } else {
            // don't change the owner; chown accepts a uid of -1
            // (unsigned) to indicate that the owner should not be
            // changed
            uid_t::max_value()
        }
    }

    fn socket_gid(&self) -> gid_t {
        // this should only be changed if the user is sudoing to a group
        // explicitly, not only if they're gaining a new primary `gid`
        if self.is_sudoing_to_explicit_group() {
            self.env.command_info.runas_egid
        } else {
            // don't change the owner; chown accepts a uid of -1
            // (unsigned) to indicate that the owner should not be
            // changed
            gid_t::max_value()
        }
    }

    fn socket_mode(&self) -> mode_t {
        // if the user is sudoing to a new `euid`, we require the
        // approver to also be able to act as the same `euid`; this is
        // the first check, because if euid changes egid is also likely
        // to change
        if self.is_sudoing_to_user() {
            return libc::S_IWUSR; // from <sys/stat.h>, writable by the user
        }

        // if the user is sudoing to a new `egid` (and not implicitly
        // by getting a new `euid`) we require the approver to also be
        // able to act as the same `egid`
        //
        // I *think* since the above statement returns only, this is
        // true if and only if `is_sudoing_to_group()` is true, but I'm
        // using the explicit version here for safety
        if self.is_sudoing_to_explicit_group() {
            return libc::S_IWGRP; // from <sys/stat.h>, writable by the group
        }

        // elsewhere, we exempt sessions for users who are sudoing to
        // themselves, so this line should never be reached; if it is,
        // it's a bug
        unreachable!("cannot determine if we're sudoing to a user or group")
    }

    fn template_spec(&self) -> Spec {
        // TODO: provide username of runas_euid?
        // TODO: provide groupname of runas_egid?
        let mut spec = Spec::with_escape(b'%');

        // the name of the appoval _b_inary
        spec.replace(b'b', self.options.binary_name());

        // the full path to the approval _B_inary
        spec.replace(b'B', self.options.binary_path.as_os_str().as_bytes());

        // the full _C_ommand `sudo` was invoked as (recreated as
        // best-effort for now)
        spec.replace(b'C', self.env.invocation());

        // the cw_d_ of the command being run under `sudo`
        spec.replace(b'd', self.env.cwd().as_os_str().as_bytes());

        // the _h_ostname of the machine `sudo` is being executed on
        spec.replace(b'h', self.env.user_info.host.as_bytes());

        // the _H_eight of the invoking user's terminal, in rows
        spec.replace(b'H', self.env.user_info.lines.to_string());

        // the real _g_id of the user invoking `sudo`
        spec.replace(b'g', self.env.user_info.gid.to_string());

        // the _p_id of this `sudo` process
        spec.replace(b'p', self.env.user_info.pid.to_string());

        // the real _u_id of the user invoking `sudo`
        spec.replace(b'u', self.env.user_info.uid.to_string());

        // the _U_sername of the user running `sudo`
        spec.replace(b'U', self.env.user_info.user.as_bytes());

        // the _W_idth of the invoking user's terminal, in columns
        spec.replace(b'W', self.env.user_info.cols.to_string());

        spec
    }
}

impl Drop for SudoPair {
    fn drop(&mut self) {
        slog::debug!(self.slog, "plugin exiting");
    }
}

#[derive(Debug)]
struct PluginOptions {
    /// `binary_path` is the location of the approval binary, so that we
    /// can bypass the approval process for invoking it.
    ///
    /// Default: `"/usr/bin/sudo_approve"`
    binary_path: PathBuf,

    /// `user_prompt_path` is the location of the prompt template to
    /// display to the user invoking sudo; if no template is found at
    /// this location, an extremely minimal default will be printed.
    ///
    /// Default: `"/etc/sudo_pair.prompt.user"`
    user_prompt_path: PathBuf,

    /// `pair_prompt_path` is the location of the prompt template to
    /// display to the user being asked to approve the sudo session; if
    /// no template is found at this location, an extremely minimal
    /// default will be printed.
    ///
    /// Default: `"/etc/sudo_pair.prompt.pair"`
    pair_prompt_path: PathBuf,

    /// `socket_dir` is the path where this plugin will store sockets for
    /// sessions that are pending approval.
    ///
    /// Default: `"/var/run/sudo_pair"`
    socket_dir: PathBuf,

    /// `gids_enforced` is a comma-separated list of gids that sudo_pair
    /// will gate access to. If a user is `sudo`ing to a user that is a
    /// member of one of these groups, they will be required to have a
    /// pair approve their session.
    ///
    /// Default: `[0]` (e.g., root)
    gids_enforced: HashSet<gid_t>,

    /// `gids_exempted` is a comma-separated list of gids whose users
    /// will be exempted from the requirements of sudo_pair. Note that
    /// this is not the opposite of the `gids_enforced` flag. Whereas
    /// `gids_enforced` gates access *to* groups, `gids_exempted`
    /// exempts users sudoing *from* groups. For instance, this setting
    /// can be used to ensure that oncall sysadmins can respond to
    /// outages without needing to find a pair.
    ///
    /// Default: `[]` (however, root is *always* exempt)
    gids_exempted: HashSet<gid_t>,
}

impl PluginOptions {
    fn binary_name(&self) -> &[u8] {
        self.binary_path.file_name().unwrap_or_else(||
            self.binary_path.as_os_str()
        ).as_bytes()
    }
}

// TODO: single_use_lifetimes was committed, but I'm not sure there's
// actually a way to satisfy the linter for the time being
#[allow(single_use_lifetimes)]
impl<'a> From<&'a OptionMap> for PluginOptions {
    fn from(map: &'a OptionMap) -> Self {
        Self {
            binary_path: map.get("binary_path")
                .unwrap_or_else(|_| DEFAULT_BINARY_PATH.into()),

            user_prompt_path: map.get("user_prompt_path")
                .unwrap_or_else(|_| DEFAULT_USER_PROMPT_PATH.into()),

            pair_prompt_path: map.get("pair_prompt_path")
                .unwrap_or_else(|_| DEFAULT_PAIR_PROMPT_PATH.into()),

            socket_dir: map.get("socket_dir")
                .unwrap_or_else(|_| DEFAULT_SOCKET_DIR.into()),

            gids_enforced: map.get("gids_enforced")
                .unwrap_or_else(|_| DEFAULT_GIDS_ENFORCED.iter().cloned().collect()),

            gids_exempted: map.get("gids_exempted")
                .unwrap_or_default(),
        }
    }
}

impl slog::Value for PluginOptions {
    fn serialize(&self, _: &slog::Record<'_>, key: slog::Key, serializer: &mut dyn slog::Serializer) -> slog::Result {
        serializer.emit_str(key, &format!("{:?}", self))
    }
}

#[cfg(all(target_os = "macos", feature = "syslog"))]
const SYSLOG_PATH: &str = "/private/var/run/syslog";

#[cfg(all(not(target_os = "macos"), feature = "syslog"))]
const SYSLOG_PATH: &str = "/dev/log";

// TODO: can we only compile slog in when logging features are enabled?
fn slog(name: &str, version: &str) -> slog::Logger {
    use slog::Drain;

    #[cfg(not(any(feature = "syslog", feature = "journald")))]
    let drain = slog::Drain::Discard;

    #[cfg(feature = "syslog")]
    let drain = slog_syslog::SyslogBuilder::new()
        .unix(SYSLOG_PATH)
        .facility(slog_syslog::Facility::LOG_AUTH)
        .start()
        .unwrap() // TODO: remove unwrap
        .ignore_res(); // TODO: handle errors

    #[cfg(feature = "journald")]
    let drain = slog_journald::JournaldDrain
        .ignore_res(); // TODO: handle errors

    slog::Logger::root(drain, slog::o!(
        "plugin_name"    => name   .to_owned(),
        "plugin_version" => version.to_owned()
    ))
}
