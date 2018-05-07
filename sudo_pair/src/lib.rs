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
// TODO: switch from error_chain to failure crate?
// TODO: error message when /var/run/sudo_pair missing
// TODO: enable the ability to respond to `sudo --version`
// TODO: iolog in `sudoreplay(8)` format
// TODO: rustfmt
// TODO: allow redirect to stdout?
// TODO: double-check all `as`-casts
// TODO: docs on docs.rs
// TODO: various badges
// TODO: fill out all fields of https://doc.rust-lang.org/cargo/reference/manifest.html
// TODO: implement change_winsize

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

// this library is fundamentally built upon unsafe code
#![allow(unsafe_code)]

#![cfg_attr(feature="cargo-clippy", warn(clippy))]
#![cfg_attr(feature="cargo-clippy", warn(clippy_pedantic))]
#![cfg_attr(feature="cargo-clippy", allow(similar_names))]

extern crate libc;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate sudo_plugin;

mod template;
mod socket;

use template::Spec;
use socket::Socket;

use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use libc::{gid_t, mode_t, uid_t};

use sudo_plugin::errors::*;
use sudo_plugin::OptionMap;

const DEFAULT_BINARY_PATH      : &str       = "/usr/bin/sudo_approve";
const DEFAULT_USER_PROMPT_PATH : &str       = "/etc/sudo_pair.prompt.user";
const DEFAULT_PAIR_PROMPT_PATH : &str       = "/etc/sudo_pair.prompt.pair";
const DEFAULT_SOCKET_DIR       : &str       = "/var/run/sudo_pair";
const DEFAULT_GIDS_ENFORCED    : [gid_t; 1] = [0];

const DEFAULT_USER_PROMPT : &[u8] = b"%B '%p %u'\n";
const DEFAULT_PAIR_PROMPT : &[u8] = b"%U@%h:%d$ %C\ny/n? [n]: ";

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
    plugin:  &'static sudo_plugin::Plugin,
    options: PluginOptions,
    socket:  Option<Socket>
}

impl SudoPair {
    fn open(plugin: &'static sudo_plugin::Plugin) -> Result<Self> {
        // TODO: convert all outgoing errors to be unauthorized errors
        let options = PluginOptions::from(&plugin.plugin_options);

        let mut pair = Self {
            plugin,
            options,
            socket: None,
        };

        if pair.is_sudoing_to_user_and_group() {
            bail!(ErrorKind::Unauthorized(
                "sudo_pair doesn't support sudoing to both a user and a group".into()
            ));
        }

        if pair.is_exempt() {
            return Ok(pair)
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

        Ok(pair)
    }

    fn close(&mut self, _: i64, _: i64) {
        // if we have a socket, close it
        let _ = self.socket.as_mut().map(|s| s.close());
    }

    fn log_ttyout(&mut self, log: &[u8]) -> Result<()> {
        // if we have a socket, write to it
        self.socket.as_mut().map_or(Ok(()), |socket| {
            socket
                .write_all(log)
                .chain_err(|| ErrorKind::Unauthorized(
                    "pair terminated the session".into()
                ))
        })
    }

    fn log_disabled(&mut self, _: &[u8]) -> Result<()> {
        // if we're exempt, don't disable stdin/stdout/stderr
        if self.is_exempt() {
            return Ok(());
        }

        bail!(ErrorKind::Unauthorized(
            "redirection of stdin, stout, and stderr prohibited".into()
        ));
    }

    fn local_pair_prompt(&self, template_spec: &Spec) {
        // read the template from the file; if there's an error, use the
        // default template instead
        let template : Vec<u8> = File::open(&self.options.user_prompt_path)
            .and_then(|file| file.bytes().collect() )
            .unwrap_or_else(|_| DEFAULT_USER_PROMPT.into() );

        let prompt = template_spec.expand(&template[..]);

        // TODO: this is returning an error (EINVAL) even though it prints
        // successfully; I'm not entirely sure why. It started failing
        // when I added some new operators for the templating code, but
        // nothing in that commit seems like it should have obviously
        // started causing writes to fail.
        //
        // EINVAL is raised by the underlying libc vfprintf call, which
        // appears to only be problematic if the underlying write fails.
        // As far as I can tell, this only happens if something isn't
        // aligned correctly and the `fd` is opened with`O_DIRECT`. But
        // it seems unlikely that STDIN is opened that way or that
        // anything Rust allocates is misaligned. The other possibility
        // is that STDIN is "unsuitable for writing" which also seems
        // improbable. For now, I'm ignoring the situation but hopefully
        // there's enough information here for someone (probably me) to
        // pick up where I left off.
        let _ = self.plugin.stdout().write(&prompt);
    }

    fn remote_pair_connect(&mut self) -> Result<()> {
        if self.socket.is_some() {
            return Ok(());
        }

        // TODO: clearly indicate when the socket path is missing
        let socket = Socket::open(
            self.socket_path(),
            self.socket_uid(),
            self.socket_gid(),
            self.socket_mode(),
        ).chain_err(|| ErrorKind::Unauthorized("unable to connect to a pair".into()))?;

        self.socket = Some(socket);

        Ok(())
    }

    fn remote_pair_prompt(&mut self, template_spec: &Spec) -> Result<()> {
        // read the template from the file; if there's an error, use the
        // default template instead
        let template : Vec<u8> = File::open(&self.options.pair_prompt_path)
            .and_then(|file| file.bytes().collect() )
            .unwrap_or_else(|_| DEFAULT_PAIR_PROMPT.into() );

        let prompt = template_spec.expand(&template[..]);

        let socket = self.socket
            .as_mut()
            .ok_or_else(|| ErrorKind::Unauthorized("unable to connect to a pair".into()))?;

        socket.write_all(&prompt[..])
            .chain_err(|| ErrorKind::Unauthorized("unable to ask pair for approval".into()))?;

        // ensure the entire prompt was written to the pair
        socket.flush()
            .chain_err(|| ErrorKind::Unauthorized("unable to ask pair for approval".into()))?;

        // default `response` to something other than success, since
        // `read` might return without actually having written anything;
        // this prevents us from being required to check the number of
        // bytes actually read from `read`
        let mut response : [u8; 1] = [b'n'];

        // read exactly one byte back from the socket for the
        // response (`read_exact` isn't used because it will capture
        // Ctrl-C and retry the read); we don't need to check the return
        // value because if the read was successful, we're guaranteed to
        // have read at least one byte
        let _ = socket.read(&mut response)
            .chain_err(|| ErrorKind::Unauthorized("denied by pair".into()))?;

        // echo back out the response, since the client is anticipated
        // to be noecho
        let _ = socket.write_all(&response[..]);
        let _ = socket.write_all(b"\n");

        match &response {
            b"y" | b"Y" => Ok(()),
            _           => Err(ErrorKind::Unauthorized("denied by pair".into()).into()),
        }
    }

    fn is_exempt(&self) -> bool {
        // root is always exempt
        if self.is_sudoing_from_root() {
            return true;
        }

        // a user sudoing entirely to themselves is weird, but I can't
        // see any reason not to let them do it without approval since
        // they can already do everything as themselves anyway
        if self.is_sudoing_to_themselves() {
            return true;
        }

        // exempt if the user who's sudoing is in a group that's exempt
        // from having to pair
        if self.is_sudoing_from_exempted_gid() {
            return true;
        }

        // exempt if none of the gids of the user we're sudoing into are
        // in the set of gids we enforce pairing for
        if !self.is_sudoing_to_enforced_gid() {
            return true;
        }

        // exempt if the approval command is the command being invoked
        if self.is_sudoing_approval_command() {
            return true;
        }

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
        self.plugin.user_info.uid == self.plugin.user_info.euid
    }

    fn is_sudoing_to_themselves(&self) -> bool {
        // if they're not sudoing to a new uid or to a new gid, they're
        // just becoming themselves... right?
        if !self.is_sudoing_to_user() && !self.is_sudoing_to_group() {
            debug_assert_eq!(
                self.plugin.runas_gids(),
                self.plugin.user_info.groups.iter().cloned().collect()
            );

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
            &self.plugin.user_info.groups.iter().cloned().collect()
        )
    }

    fn is_sudoing_to_enforced_gid(&self) -> bool {
        !self.options.gids_enforced.is_disjoint(
            &self.plugin.runas_gids()
        )
    }

    fn is_sudoing_approval_command(&self) -> bool {
        self.plugin.command_info.command == self.options.binary_path
    }

    fn is_sudoing_to_user(&self) -> bool {
        self.plugin.user_info.uid != self.plugin.command_info.runas_euid
    }

    fn is_sudoing_to_group(&self) -> bool {
        self.plugin.user_info.gid != self.plugin.command_info.runas_egid
    }

    // returns true if `-g` was specified
    fn is_sudoing_to_explicit_group(&self) -> bool {
        self.plugin.settings.runas_group.is_some()
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
                self.plugin.user_info.uid,
                self.plugin.user_info.pid,
            )
        )
    }

    fn socket_uid(&self) -> uid_t {
        // we explicitly want to have the socket owned by the root user
        // if we're doing `sudo -g`, so that the sudoing user can't
        // silently self-approve by manually connecting to the socket
        // without needing to invoke sudo
        if self.is_sudoing_to_user() {
            self.plugin.command_info.runas_euid
        } else {
            // don't change the owner; chown accepts a uid of -1
            // (unsigned) to indicate that the owner should not be
            // changed
            uid_t::max_value()
        }
    }

    fn socket_gid(&self) -> gid_t {
        // it's probably unnecessary to use our own gid in the event of
        // sudoing to the same group, since the mode should be set
        // correctly either way, but I'm doing so anyway in the interest
        // of caution
        if self.is_sudoing_to_group() {
            self.plugin.command_info.runas_egid
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

        // if the user is sudoing to a new `egid`, we require the
        // approver to also be able to act as the same `egid`
        if self.is_sudoing_to_group() {
            return libc::S_IWGRP; // from <sys/stat.h>, writable by the group
        }

        // elsewhere, we exempt sessions for users who are sudoing to
        // themselves, so this line should never be reached; if it is,
        // it's a bug
        unreachable!("cannot determine if we're sudoing to a user or group")
    }

    fn template_spec(&self) -> Spec {
        // TODO: document these somewhere useful for users of this plugin
        // TODO: provide groupname of gid?
        // TODO: provide username of runas_euid?
        // TODO: provide groupname of runas_egid?
        let mut spec = Spec::with_escape(b'%');

        // the name of the appoval _b_inary
        spec.replace(b'b', self.options.binary_name());

        // the full path to the approval _B_inary
        spec.replace(b'B', self.options.binary_path.as_os_str().as_bytes());

        // the full _C_ommand `sudo` was invoked as (recreated as
        // best-effort for now)
        spec.replace(b'C', self.plugin.invocation());

        // the cw_d_ of the command being run under `sudo`
        spec.replace(b'd', self.plugin.cwd().as_os_str().as_bytes());

        // the _h_ostname of the machine `sudo` is being executed on
        spec.replace(b'h', self.plugin.user_info.host.as_bytes());

        // the _H_eight of the invoking user's terminal, in rows
        spec.replace(b'H', self.plugin.user_info.lines.to_string());

        // the real _g_id of the user invoking `sudo`
        spec.replace(b'g', self.plugin.user_info.gid.to_string());

        // the _p_id of this `sudo` process
        spec.replace(b'p', self.plugin.user_info.pid.to_string());

        // the real _u_id of the user invoking `sudo`
        spec.replace(b'u', self.plugin.user_info.uid.to_string());

        // the _U_sername of the user running `sudo`
        spec.replace(b'U', self.plugin.user_info.user.as_bytes());

        // the _W_idth of the invoking user's terminal, in columns
        spec.replace(b'W', self.plugin.user_info.cols.to_string());

        spec
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
