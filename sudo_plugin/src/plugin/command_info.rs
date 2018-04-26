use super::super::errors::*;
use super::option_map::*;

use std::os::unix::io::RawFd;
use std::path::PathBuf;

use libc::{gid_t, mode_t, uid_t};

#[derive(Debug)]
pub struct CommandInfo {
    pub chroot:            Option<String>,
    pub close_from:        Option<u64>,
    pub command:           PathBuf,
    pub cwd:               Option<PathBuf>,
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
    pub fn try_from(value: OptionMap) -> Result<Self> {
        Ok(Self {
            command:       value.get("command")?,
            runas_gid:     value.get("runas_gid")?,
            runas_uid:     value.get("runas_uid")?,
            runas_egid:    value.get("runas_egid")
                .unwrap_or(value.get("runas_gid")?),
            runas_euid:    value.get("runas_euid")
                .unwrap_or(value.get("runas_uid")?),
            umask:         value.get("umask")?,

            chroot:            value.get("chroot")            .ok(),
            close_from:        value.get("closefrom")         .ok(),
            cwd:               value.get("cwd")               .ok(),
            exec_background:   value.get("exec_background")   .unwrap_or(false),
            exec_fd:           value.get("execfd")            .ok(),
            iolog_compress:    value.get("iolog_compress")    .unwrap_or(false),
            iolog_path:        value.get("iolog_path")        .ok(),
            iolog_stdin:       value.get("iolog_stdin")       .unwrap_or(false),
            iolog_stdout:      value.get("iolog_stdout")      .unwrap_or(false),
            iolog_stderr:      value.get("iolog_stderr")      .unwrap_or(false),
            iolog_ttyin:       value.get("iolog_ttyin")       .unwrap_or(false),
            iolog_ttyout:      value.get("iolog_ttyout")      .unwrap_or(false),
            login_class:       value.get("login_class")       .ok(),
            nice:              value.get("nice")              .ok(),
            noexec:            value.get("noexec")            .unwrap_or(false),
            preserve_fds:      value.get("preserve_fds")      .unwrap_or_else(|_| vec![]),
            preserve_groups:   value.get("preserve_groups")   .unwrap_or(false),
            runas_groups:      value.get("runas_groups")      .ok(),
            selinux_role:      value.get("selinux_role")      .ok(),
            selinux_type:      value.get("selinux_type")      .ok(),
            set_utmp:          value.get("set_utmp")          .unwrap_or(false),
            sudoedit:          value.get("sudoedit")          .unwrap_or(false),
            sudoedit_checkdir: value.get("sudoedit_checkdir") .unwrap_or(true),
            sudoedit_follow:   value.get("sudoedit_follow")   .unwrap_or(false),
            timeout:           value.get("timeout")           .ok(),
            use_pty:           value.get("use_pty")           .unwrap_or(false),
            utmp_user:         value.get("utmp_user")         .ok(),

            raw: value,
        })
    }
}
