use super::super::errors::*;
use super::option_map::*;

use std::os::unix::io::RawFd;

use libc::{gid_t, mode_t, uid_t};

#[derive(Debug)]
pub struct CommandInfo {
    pub chroot:            Option<String>,
    pub close_from:        Option<u64>,
    pub command:           String,
    pub cwd:               Option<String>,
    pub exec_background:   bool,
    pub exec_fd:           Option<u64>,
    pub iolog_compress:    bool,
    pub iolog_path:        Option<String>,
    pub iolog_stdin:       bool,
    pub iolog_stdout:      bool,
    pub iolog_stderr:      bool,
    pub iolog_ttyin:       bool,
    pub iolog_ttyout:      bool,
    pub login_class:       Option<String>,
    pub nice:              Option<u64>,
    pub noexec:            bool,
    pub preserve_fds:      Vec<RawFd>,
    pub preserve_groups:   bool,
    pub runas_egid:        gid_t,
    pub runas_euid:        uid_t,
    pub runas_gid:         gid_t,
    pub runas_groups:      Option<Vec<gid_t>>,
    pub runas_uid:         uid_t,
    pub selinux_role:      Option<String>,
    pub selinux_type:      Option<String>,
    pub set_utmp:          bool,
    pub sudoedit:          bool,
    pub sudoedit_checkdir: bool,
    pub sudoedit_follow:   bool,
    pub timeout:           Option<u64>,
    pub umask:             mode_t,
    pub use_pty:           bool,
    pub utmp_user:         Option<String>,

    pub raw: OptionMap,
}

impl CommandInfo {
    pub fn new(raw: OptionMap) -> Result<Self> {
        Ok(CommandInfo {
            command:       raw.get_parsed("command")?,
            runas_gid:     raw.get_parsed("runas_gid")?,
            runas_uid:     raw.get_parsed("runas_uid")?,
            runas_egid:    raw.get_parsed("runas_egid")
                .unwrap_or(raw.get_parsed("runas_gid")?),
            runas_euid:    raw.get_parsed("runas_euid")
                .unwrap_or(raw.get_parsed("runas_uid")?),
            umask:         raw.get_parsed("umask")?,

            chroot:            raw.get_parsed("chroot")            .ok(),
            close_from:        raw.get_parsed("closefrom")         .ok(),
            cwd:               raw.get_parsed("cwd")               .ok(),
            exec_background:   raw.get_parsed("exec_background")   .unwrap_or(false),
            exec_fd:           raw.get_parsed("execfd")            .ok(),
            iolog_compress:    raw.get_parsed("iolog_compress")    .unwrap_or(false),
            iolog_path:        raw.get_parsed("iolog_path")        .ok(),
            iolog_stdin:       raw.get_parsed("iolog_stdin")       .unwrap_or(false),
            iolog_stdout:      raw.get_parsed("iolog_stdout")      .unwrap_or(false),
            iolog_stderr:      raw.get_parsed("iolog_stderr")      .unwrap_or(false),
            iolog_ttyin:       raw.get_parsed("iolog_ttyin")       .unwrap_or(false),
            iolog_ttyout:      raw.get_parsed("iolog_ttyout")      .unwrap_or(false),
            login_class:       raw.get_parsed("login_class")       .ok(),
            nice:              raw.get_parsed("nice")              .ok(),
            noexec:            raw.get_parsed("noexec")            .unwrap_or(false),
            preserve_fds:      raw.get_parsed("preserve_fds")      .unwrap_or(vec![]),
            preserve_groups:   raw.get_parsed("preserve_groups")   .unwrap_or(false),
            runas_groups:      raw.get_parsed("runas_groups")      .ok(),
            selinux_role:      raw.get_parsed("selinux_role")      .ok(),
            selinux_type:      raw.get_parsed("selinux_type")      .ok(),
            set_utmp:          raw.get_parsed("set_utmp")          .unwrap_or(false),
            sudoedit:          raw.get_parsed("sudoedit")          .unwrap_or(false),
            sudoedit_checkdir: raw.get_parsed("sudoedit_checkdir") .unwrap_or(true),
            sudoedit_follow:   raw.get_parsed("sudoedit_follow")   .unwrap_or(false),
            timeout:           raw.get_parsed("timeout")           .ok(),
            use_pty:           raw.get_parsed("use_pty")           .unwrap_or(false),
            utmp_user:         raw.get_parsed("utmp_user")         .ok(),

            raw: raw,
        })
    }
}
