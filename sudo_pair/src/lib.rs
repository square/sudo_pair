//! sudo IO-plugin to require a live human pair.
//!
//! TODO: explain

// TODO: remove all unwraps
// TODO: remove all to_string_lossy
// TODO: switch from error_chain to failure crate?
// TODO: error message when /var/run/sudo_pair missing
// TODO: enable the ability to respond to `sudo --version`

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
extern crate unix_socket;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate sudo_plugin;

mod socket;

use socket::Socket;

use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{PathBuf, Path};

use libc::{gid_t, mode_t, uid_t};

use sudo_plugin::errors::*;
use sudo_plugin::OptionMap;

const DEFAULT_BINARY_PATH      : &str = "/usr/bin/sudo_pair_approve";
const DEFAULT_USER_PROMPT_PATH : &str = "/etc/sudo_pair.prompt.user";
const DEFAULT_PAIR_PROMPT_PATH : &str = "/etc/sudo_pair.prompt.pair";
const DEFAULT_SOCKET_DIR       : &str = "/var/run/sudo_pair";

const TEMPLATE_ESCAPE     : u8    = b'%';
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
    plugin:      &'static sudo_plugin::Plugin,
    settings:    PluginSettings,
    socket:      Option<Socket>
}

impl SudoPair {
    fn open(plugin: &'static sudo_plugin::Plugin) -> Result<Self> {
        // TODO: convert all outgoing errors to be unauthorized errors
        let settings    = PluginSettings::from(&plugin.plugin_options);

        let mut pair = Self {
            plugin,
            settings,
            socket: None,
        };

        if !pair.is_exempt() {
            pair.local_pair_prompt()?;
            pair.remote_pair_connect()?;
            pair.remote_pair_prompt()?;
        }

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

    fn local_pair_prompt(&self) -> Result<()> {
        // read the template from the file; if there's an error, use the
        // default template instead
        let template = self.template_load(
            &self.settings.user_prompt_path
        ).unwrap_or_else(|_| DEFAULT_USER_PROMPT.to_owned());

        // TODO: this is returning an error even though it printf, I'm not
        // entirely sure why
        let _ = self.plugin.print_info(
            self.template_expand(&template[..])
        );

        Ok(())
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

    fn remote_pair_prompt(&mut self) -> Result<()> {
        let template = self.template_load(
            &self.settings.pair_prompt_path
        ).unwrap_or_else(|_| DEFAULT_PAIR_PROMPT.to_owned());

        let prompt = self.template_expand(&template[..]);

        let socket = self.socket
            .as_mut()
            .ok_or_else(|| ErrorKind::Unauthorized("unable to connect to a pair".into()))?;

        let mut response : [u8; 1] = unsafe {
            ::std::mem::uninitialized()
        };

        let _ = socket.write(&prompt[..])
            .chain_err(|| ErrorKind::Unauthorized("unable to ask pair for approval".into()))?;

        // read exactly one byte back from the socket for the
        // response
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
        self.plugin.user_info.uid == unsafe { libc::geteuid() }
    }

    fn is_sudoing_from_exempted_gid(&self) -> bool {
        !self.settings.gids_exempted.is_disjoint(
            &self.plugin.user_info.groups.iter().cloned().collect()
        )
    }

    fn is_sudoing_to_enforced_gid(&self) -> bool {
        !self.settings.gids_enforced.is_disjoint(
            &self.plugin.runas_gids().iter().cloned().collect()
        )
    }

    fn is_sudoing_approval_command(&self) -> bool {
        self.plugin.command_info.command == self.settings.binary_path
    }

    fn is_sudoing_to_user(&self) -> bool {
        self.plugin.user_info.euid != self.plugin.command_info.runas_euid
    }

    fn is_sudoing_to_group(&self) -> bool {
        self.plugin.user_info.egid != self.plugin.command_info.runas_egid
    }

    fn socket_path(&self) -> PathBuf {
        // we encode the originating `uid` into the pathname since
        // there's no other (easy) way for the approval command to probe
        // for this information
        self.settings.socket_dir.join(
            format!(
                "{}.{}.sock",
                self.plugin.user_info.euid,
                self.plugin.user_info.pid
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
            // the *effective* uid is the one we want here since it's
            // the uid of the elevated `sudo` process; `getuid` would
            // return the invoking user's uid (ask me how I noticed
            // this)
            unsafe { libc::geteuid() }
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
            // the *effective* gid is the one we want here since it's
            // the gid of the elevated `sudo` process; `getgid` would
            // return the invoking user's gid (ask me how I noticed
            // this)
            unsafe { libc::getegid() }
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

        // this is here as a fallback in case of an unexpected sudo
        // invocation that we don't know how to support; if you're
        // sudoing to yourself, as yourself... maybe the command should
        // be exempted, but for now I'm erring on the side of caution
        //
        // TODO: I actually hit this during testing (sudoing to myself),
        // so I should consider what to actually do about this
        unreachable!()
    }

    fn template_load(&self, path: &Path) -> ::std::io::Result<Vec<u8>> {
        let mut template = vec![];

        File::open(path).and_then(|mut f|
            f.read_to_end(&mut template)
        ).map(|_| template)
    }

    fn template_expand(&self, template : &[u8]) -> Vec<u8> {
        let mut result = vec![];
        let mut iter   = template.iter().cloned();

        while iter.len() != 0 {
            // copy everything up to the next %-sign unchanged
            result.extend(iter.by_ref().take_while(|b| *b != TEMPLATE_ESCAPE ));

            if iter.len() == 0 {
                break;
            }

            // we expand each literal into an owned type so that we don't have
            // to repeatd the `result.extend_from_slice` part each time in the
            // match arms, but it does kind of suck that we have so much
            // type-conversion noise
            let expansion = match iter.next() {
                Some(b'b') => self.settings.binary_name().into(),
                Some(b'B') => self.settings.binary_path.as_os_str().as_bytes().into(),
                Some(b'C') => self.plugin.invocation(),
                Some(b'd') => self.plugin.cwd().as_os_str().as_bytes().into(),
                Some(b'h') => self.plugin.user_info.host.as_bytes().into(),
                Some(b'H') => self.plugin.user_info.lines.to_string().into_bytes(),
                Some(b'g') => self.plugin.user_info.egid.to_string().into_bytes(),
                Some(b'p') => self.plugin.user_info.pid.to_string().into_bytes(),
                Some(b'u') => self.plugin.user_info.euid.to_string().into_bytes(),
                Some(b'U') => self.plugin.user_info.user.as_bytes().into(),
                Some(b'W') => self.plugin.user_info.cols.to_string().into_bytes(),
                Some(byte) => vec![TEMPLATE_ESCAPE, byte],
                None       => vec![TEMPLATE_ESCAPE],
            };

            result.extend_from_slice(&expansion[..]);
        }

        result
    }
}

#[derive(Debug)]
struct PluginSettings {
    /// `binary_path` is the location of the approval binary, so that we
    /// can bypass the approval process for invoking it
    ///
    /// Default: `"/usr/bin/sudo_pair_approve"`
    binary_path: PathBuf,

    /// `user_prompt_path` is the location of the prompt template to display
    /// to the user invoking sudo; if no template is found at this location,
    /// an extremely minimal default will be printed
    ///
    /// Default: `"/etc/sudo_pair.prompt.user"`
    user_prompt_path: PathBuf,

    /// `pair_prompt_path` is the location of the prompt template to display
    /// to the user being asked to approve the sudo session sudo; if no
    /// template is found at this location, an extremely minimal default will
    /// be printed
    ///
    /// Default: `"/etc/sudo_pair.prompt.pair"`
    pair_prompt_path: PathBuf,

    /// `socket_dir` is the path where this plugin will store sockets for
    /// sessions that are pending approval
    ///
    /// Default: `"/var/run/sudo_pair"`
    socket_dir: PathBuf,

    // TODO: doc
    gids_enforced: HashSet<gid_t>,
    gids_exempted: HashSet<gid_t>,
}

impl PluginSettings {
    fn binary_name(&self) -> &[u8] {
        self.binary_path.file_name().unwrap_or_else(||
            self.binary_path.as_os_str()
        ).as_bytes()
    }
}

impl<'a> From<&'a OptionMap> for PluginSettings {
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

            gids_enforced: map.get::<Vec<gid_t>>("gids_enforced")
                .unwrap_or_default().iter().cloned().collect(),

            gids_exempted: map.get::<Vec<gid_t>>("gids_exempted")
                .unwrap_or_default().iter().cloned().collect(),
        }
    }
}
