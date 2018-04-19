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
use std::ffi::CStr;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use libc::{gid_t, mode_t, pid_t, uid_t};

use sudo_plugin::errors::*;
use sudo_plugin::OptionMap;

const DEFAULT_BINARY_PATH : &str = "/usr/bin/sudo_pair_approve";
const DEFAULT_PROMPT_PATH : &str = "/etc/sudo_pair.prompt";
const DEFAULT_SOCKET_DIR  : &str = "/var/run/sudo_pair";

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
    environment: PluginEnvironment,
    socket:      Option<Socket>
}

impl SudoPair {
    fn open(plugin: &'static sudo_plugin::Plugin) -> Result<Self> {
        // TODO: convert all outgoing errors to be unauthorized errors
        let settings    = PluginSettings::from(&plugin.plugin_options);
        let environment = PluginEnvironment::new(plugin)?;

        println!("{:#?}", plugin.user_info);
        println!("{:#?}", settings);
        println!("{:#?}", environment);

        let mut pair = Self {
            plugin,
            settings,
            environment,
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
        // TODO: configurable file path
        // TODO: error handling
        let mut template         = String::new();
        let mut prompt : Vec<u8> = vec![];

        File::open(&self.settings.prompt_path).map(|mut file| {
            let _ = file.read_to_string(&mut template);
        }).unwrap_or_else(|_| {
            template.push_str("%b %p %u");
        });

        let mut iter = template.bytes().into_iter().peekable();

        while iter.peek().is_some() {
            let fragment : Vec<u8> = iter.by_ref().take_while(|b| *b != b'%' ).collect();
            prompt.extend_from_slice(&fragment[..]);

            match iter.next() {
                Some(b'b') => prompt.extend_from_slice(self.settings.binary_path.as_os_str().as_bytes()),
                Some(b'h') => prompt.extend_from_slice(self.environment.hostname.as_ref()),
                Some(b'p') => prompt.extend_from_slice(self.environment.pid.to_string().as_ref()),
                Some(b'u') => prompt.extend_from_slice(self.environment.uid.to_string().as_ref()),
                Some(x)    => prompt.push(x),
                None       => (),
            };
        }

        let _ = self.plugin.print_info(prompt)?;

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
        let socket = self.socket
            .as_mut()
            .ok_or_else(|| ErrorKind::Unauthorized("unable to connect to a pair".into()))?;

        let mut response : [u8; 1] = unsafe {
            ::std::mem::uninitialized()
        };

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
        self.environment.uid == 0
    }

    fn is_sudoing_from_exempted_gid(&self) -> bool {
        !self.settings.gids_exempted.is_disjoint(
            &self.environment.gids
        )
    }

    fn is_sudoing_to_enforced_gid(&self) -> bool {
        !self.settings.gids_enforced.is_disjoint(
            &self.environment.runas_gids
        )
    }

    fn is_sudoing_approval_command(&self) -> bool {
        self.environment.command == self.settings.binary_path
    }

    fn is_sudoing_to_user(&self) -> bool {
        self.environment.uid != self.environment.runas_uid
    }

    fn is_sudoing_to_group(&self) -> bool {
        self.environment.gid != self.environment.runas_gid
    }

    fn socket_path(&self) -> PathBuf {
        // we encode the originating `uid` into the pathname since
        // there's no other (easy) way for the approval command to probe
        // for this information
        self.settings.socket_dir.join(
            format!("{}.{}.sock", self.environment.uid, self.environment.pid)
        )
    }

    fn socket_uid(&self) -> uid_t {
        // we explicitly want to have the socket owned by the root user
        // if we're doing `sudo -g`, so that the sudoing user can't
        // silently self-approve by manually connecting to the socket
        // without needing to invoke sudo
        if self.is_sudoing_to_user() {
            self.environment.runas_uid
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
            self.environment.runas_gid
        } else {
            unsafe { libc::getgid() } // just in case we're not gid 0
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
        unreachable!()
    }
}

#[derive(Debug)]
struct PluginEnvironment {
    /// The hostname of the machine the command is being invoked on.
    hostname: String,

    /// The uid of the user invoking the command.
    uid: uid_t,

    /// The primary gid of the user invoking the command.
    gid: gid_t,

    /// The gids of the user invoking the command.
    gids: HashSet<gid_t>,

    /// The username of the user invoking the command.
    username: String,

    /// The process ID of the `sudo` invocation.
    pid: pid_t,

    /// The fully qualified path to the command to be executed.
    // TODO: use the full args too
    command: PathBuf,

    /// The current working directory to change to when executing the
    /// command.
    cwd: PathBuf,

    /// The elevated user ID the command is being invoked under.
    runas_uid: uid_t,

    /// The elevated group ID the command is being invoked under.
    runas_gid: gid_t,

    /// The full set of group memberships the command will be run with.
    runas_gids: HashSet<gid_t>,

    /// The username of the elevated user ID the command is being invoked
    /// under.
    runas_username: String,

    /// The groupname of the elevated group ID the command is being invoked
    /// under.
    runas_groupname: String,
}

fn uid_to_username(uid: uid_t) -> Result<String> {
    let pwent = unsafe {
        libc::getpwuid(uid)
    };

    if pwent.is_null() {
        bail!(ErrorKind::Unauthorized("target user couldn't be found".into()))
    }

    unsafe {
        Ok(
            CStr::from_ptr((*pwent).pw_name)
                .to_str()
                .chain_err(|| "target user couldn't be found")?
                .to_owned()
        )
    }
}

fn gid_to_groupname(gid: gid_t) -> Result<String> {
    let pwent = unsafe {
        libc::getgrgid(gid)
    };

    if pwent.is_null() {
        bail!(ErrorKind::Unauthorized("target group couldn't be found".into()))
    }

    unsafe {
        Ok(
            CStr::from_ptr((*pwent).gr_name)
                .to_str()
                .chain_err(|| "target group couldn't be found")?
                .to_owned()
        )
    }
}

impl<'a> PluginEnvironment {
    fn new(plugin: &'a sudo_plugin::Plugin) -> Result<Self> {
        let gids : HashSet<gid_t> = plugin.user_info.groups
            .iter()
            .cloned()
            .collect();

        // if -P is passed to `sudo`, `runas_groups` is empty, but the
        // effective groups are the user's original ones
        let runas_gids = plugin.command_info.runas_groups
            .as_ref()
            .map_or_else(
                || gids.clone(),
                |gids| gids.iter().cloned().collect()
            );

        Ok(Self {
            hostname:        plugin.user_info.host.clone(),
            uid:             plugin.user_info.uid,
            gid:             plugin.user_info.gid,
            gids,
            username:        plugin.user_info.user.clone(),
            pid:             plugin.user_info.pid,
            command:         PathBuf::from(&plugin.command_info.command),
            cwd:             PathBuf::from(plugin.command_info.cwd.as_ref().unwrap_or(&plugin.user_info.cwd)),
            runas_uid:       plugin.command_info.runas_uid,
            runas_gid:       plugin.command_info.runas_gid,
            runas_gids,
            runas_username:  uid_to_username(plugin.command_info.runas_uid)?,
            runas_groupname: gid_to_groupname(plugin.command_info.runas_gid)?,
        })
    }
}

#[derive(Debug)]
struct PluginSettings {
    /// `binary_path` is the location of the approval binary, so that we
    /// can bypass the approval process for invoking it
    ///
    /// Default: `"/usr/bin/sudo_pair_approve"`
    binary_path: PathBuf,

    /// `prompt_path` is the location of a prompt tempalate; if no
    /// template is found at this location, an extremely minimal default
    /// will be printed
    ///
    /// Default: `"/etc/sudo_pair.prompt"`
    prompt_path: PathBuf,

    /// `socket_dir` is the path where this plugin will store sockets for
    /// sessions that are pending approval
    ///
    /// Default: `"/var/run/sudo_pair"`
    socket_dir: PathBuf,

    // TODO: doc
    gids_enforced: HashSet<gid_t>,
    gids_exempted: HashSet<gid_t>,
}

impl<'a> From<&'a OptionMap> for PluginSettings {
    fn from(map: &'a OptionMap) -> Self {
        Self {
            binary_path: map.get_parsed("binary_path")
                .unwrap_or_else(|_| DEFAULT_BINARY_PATH.into()),

            prompt_path: map.get_parsed("prompt_path")
                .unwrap_or_else(|_| DEFAULT_PROMPT_PATH.into()),

            socket_dir: map.get_parsed("socket_dir")
                .unwrap_or_else(|_| DEFAULT_SOCKET_DIR.into()),

            gids_enforced: map.get_parsed("gids_enforced")
                .unwrap_or_else(|_| vec![]).into_iter().collect(),

            gids_exempted: map.get_parsed("gids_exempted")
                .unwrap_or_else(|_| vec![]).into_iter().collect(),
        }
    }
}
