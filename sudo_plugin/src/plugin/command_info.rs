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

use super::super::errors::*;
use super::parsing;

use std::collections::HashMap;
use std::ffi::OsString;
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

    pub raw: HashMap<OsString, OsString>,
}

impl CommandInfo {
   pub fn new(ptr: *const *const c_char) -> Result<Self> {
        let raw = unsafe {
            parsing::parse_options(ptr)
        }?;

        Ok(CommandInfo {
            chroot:            parsing::parse_raw(&raw, "chroot",            parsing::parse)    .ok(),
            close_from:        parsing::parse_raw(&raw, "close_from",        parsing::parse)    .ok(),
            command:           parsing::parse_raw(&raw, "command",           parsing::parse)?,
            cwd:               parsing::parse_raw(&raw, "cwd",               parsing::parse)    .ok(),
            exec_background:   parsing::parse_raw(&raw, "exec_background",   parsing::parse)    .unwrap_or(false),
            exec_fd:           parsing::parse_raw(&raw, "exec_fd",           parsing::parse)    .ok(),
            iolog_compress:    parsing::parse_raw(&raw, "iolog_compress",    parsing::parse)    .unwrap_or(false),
            iolog_path:        parsing::parse_raw(&raw, "iolog_path",        parsing::parse)    .ok(),
            iolog_stdin:       parsing::parse_raw(&raw, "iolog_stdin",       parsing::parse)    .unwrap_or(false),
            iolog_stdout:      parsing::parse_raw(&raw, "iolog_stdout",      parsing::parse)    .unwrap_or(false),
            iolog_stderr:      parsing::parse_raw(&raw, "iolog_stderr",      parsing::parse)    .unwrap_or(false),
            iolog_ttyin:       parsing::parse_raw(&raw, "iolog_ttyin",       parsing::parse)    .unwrap_or(false),
            iolog_ttyout:      parsing::parse_raw(&raw, "iolog_ttyout",      parsing::parse)    .unwrap_or(false),
            login_class:       parsing::parse_raw(&raw, "login_class",       parsing::parse)    .ok(),
            nice:              parsing::parse_raw(&raw, "nice",              parsing::parse)    .ok(),
            noexec:            parsing::parse_raw(&raw, "noexec",            parsing::parse)    .unwrap_or(false),
            preserve_fds:      parsing::parse_raw(&raw, "preserve_fds",      parsing::parse_fds).unwrap_or(vec![]),
            preserve_groups:   parsing::parse_raw(&raw, "preserve_groups",   parsing::parse)    .unwrap_or(false),
            runas_egid:        parsing::parse_raw(&raw, "runas_egid",        parsing::parse)    .unwrap_or(
                               parsing::parse_raw(&raw, "runas_gid",         parsing::parse)?),
            runas_euid:        parsing::parse_raw(&raw, "runas_euid",        parsing::parse)    .unwrap_or(
                               parsing::parse_raw(&raw, "runas_uid",         parsing::parse)?),
            runas_gid:         parsing::parse_raw(&raw, "runas_gid",         parsing::parse)?,
            runas_groups:      parsing::parse_raw(&raw, "runas_groups",      parsing::parse_gids)?,
            runas_uid:         parsing::parse_raw(&raw, "runas_uid",         parsing::parse)?,
            selinux_role:      parsing::parse_raw(&raw, "selinux_role",      parsing::parse)    .ok(),
            selinux_type:      parsing::parse_raw(&raw, "selinux_type",      parsing::parse)    .ok(),
            set_utmp:          parsing::parse_raw(&raw, "set_utmp",          parsing::parse)    .unwrap_or(false),
            sudoedit:          parsing::parse_raw(&raw, "sudoedit",          parsing::parse)    .unwrap_or(false),
            sudoedit_checkdir: parsing::parse_raw(&raw, "sudoedit_checkdir", parsing::parse)    .unwrap_or(true),
            sudoedit_follow:   parsing::parse_raw(&raw, "sudoedit_follow",   parsing::parse)    .unwrap_or(false),
            timeout:           parsing::parse_raw(&raw, "timeout",           parsing::parse)    .ok(),
            umask:             parsing::parse_raw(&raw, "umask",             parsing::parse)?,
            use_pty:           parsing::parse_raw(&raw, "use_pty",           parsing::parse)    .unwrap_or(false),
            utmp_user:         parsing::parse_raw(&raw, "utmp_user",         parsing::parse)    .ok(),

            raw: raw,
        })
   }
}
