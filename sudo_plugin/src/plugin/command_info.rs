use super::super::errors::*;
use super::parsing;

use std::collections::HashMap;
use std::ffi::CString;
use std::os::unix::io::RawFd;

use libc::{c_char, gid_t, mode_t, uid_t};

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
    pub runas_groups:      Vec<gid_t>,
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

    pub raw: HashMap<CString, CString>,
}

impl CommandInfo {
   pub fn new(ptr: *const *const c_char) -> Result<Self> {
        let raw = unsafe {
            parsing::parse_options(ptr)
        }?;

        Ok(CommandInfo {
            command:       parsing::parse_raw(&raw, b"command\0",      parsing::parse)?,
            runas_gid:     parsing::parse_raw(&raw, b"runas_gid\0",    parsing::parse)?,
            runas_groups:  parsing::parse_raw(&raw, b"runas_groups\0", parsing::parse_gids)?,
            runas_uid:     parsing::parse_raw(&raw, b"runas_uid\0",    parsing::parse)?,
            runas_egid:    parsing::parse_raw(&raw, b"runas_egid\0",   parsing::parse)
                .unwrap_or(parsing::parse_raw(&raw, b"runas_gid\0",    parsing::parse)?),
            runas_euid:    parsing::parse_raw(&raw, b"runas_euid\0",   parsing::parse)
                .unwrap_or(parsing::parse_raw(&raw, b"runas_uid\0",    parsing::parse)?),
            umask:         parsing::parse_raw(&raw, b"umask\0",        parsing::parse)?,

            chroot:            parsing::parse_raw(&raw, b"chroot\0",            parsing::parse)    .ok(),
            close_from:        parsing::parse_raw(&raw, b"close_from\0",        parsing::parse)    .ok(),
            cwd:               parsing::parse_raw(&raw, b"cwd\0",               parsing::parse)    .ok(),
            exec_background:   parsing::parse_raw(&raw, b"exec_background\0",   parsing::parse)    .unwrap_or(false),
            exec_fd:           parsing::parse_raw(&raw, b"exec_fd\0",           parsing::parse)    .ok(),
            iolog_compress:    parsing::parse_raw(&raw, b"iolog_compress\0",    parsing::parse)    .unwrap_or(false),
            iolog_path:        parsing::parse_raw(&raw, b"iolog_path\0",        parsing::parse)    .ok(),
            iolog_stdin:       parsing::parse_raw(&raw, b"iolog_stdin\0",       parsing::parse)    .unwrap_or(false),
            iolog_stdout:      parsing::parse_raw(&raw, b"iolog_stdout\0",      parsing::parse)    .unwrap_or(false),
            iolog_stderr:      parsing::parse_raw(&raw, b"iolog_stderr\0",      parsing::parse)    .unwrap_or(false),
            iolog_ttyin:       parsing::parse_raw(&raw, b"iolog_ttyin\0",       parsing::parse)    .unwrap_or(false),
            iolog_ttyout:      parsing::parse_raw(&raw, b"iolog_ttyout\0",      parsing::parse)    .unwrap_or(false),
            login_class:       parsing::parse_raw(&raw, b"login_class\0",       parsing::parse)    .ok(),
            nice:              parsing::parse_raw(&raw, b"nice\0",              parsing::parse)    .ok(),
            noexec:            parsing::parse_raw(&raw, b"noexec\0",            parsing::parse)    .unwrap_or(false),
            preserve_fds:      parsing::parse_raw(&raw, b"preserve_fds\0",      parsing::parse_fds).unwrap_or(vec![]),
            preserve_groups:   parsing::parse_raw(&raw, b"preserve_groups\0",   parsing::parse)    .unwrap_or(false),
            selinux_role:      parsing::parse_raw(&raw, b"selinux_role\0",      parsing::parse)    .ok(),
            selinux_type:      parsing::parse_raw(&raw, b"selinux_type\0",      parsing::parse)    .ok(),
            set_utmp:          parsing::parse_raw(&raw, b"set_utmp\0",          parsing::parse)    .unwrap_or(false),
            sudoedit:          parsing::parse_raw(&raw, b"sudoedit\0",          parsing::parse)    .unwrap_or(false),
            sudoedit_checkdir: parsing::parse_raw(&raw, b"sudoedit_checkdir\0", parsing::parse)    .unwrap_or(true),
            sudoedit_follow:   parsing::parse_raw(&raw, b"sudoedit_follow\0",   parsing::parse)    .unwrap_or(false),
            timeout:           parsing::parse_raw(&raw, b"timeout\0",           parsing::parse)    .ok(),
            use_pty:           parsing::parse_raw(&raw, b"use_pty\0",           parsing::parse)    .unwrap_or(false),
            utmp_user:         parsing::parse_raw(&raw, b"utmp_user\0",         parsing::parse)    .ok(),

            raw: raw,
        })
   }
}
