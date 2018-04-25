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
        Ok(Self {
            command:       raw.get("command")?,
            runas_gid:     raw.get("runas_gid")?,
            runas_uid:     raw.get("runas_uid")?,
            runas_egid:    raw.get("runas_egid")
                .unwrap_or(raw.get("runas_gid")?),
            runas_euid:    raw.get("runas_euid")
                .unwrap_or(raw.get("runas_uid")?),
            umask:         raw.get("umask")?,

            chroot:            raw.get("chroot")            .ok(),
            close_from:        raw.get("closefrom")         .ok(),
            cwd:               raw.get("cwd")               .ok(),
            exec_background:   raw.get("exec_background")   .unwrap_or(false),
            exec_fd:           raw.get("execfd")            .ok(),
            iolog_compress:    raw.get("iolog_compress")    .unwrap_or(false),
            iolog_path:        raw.get("iolog_path")        .ok(),
            iolog_stdin:       raw.get("iolog_stdin")       .unwrap_or(false),
            iolog_stdout:      raw.get("iolog_stdout")      .unwrap_or(false),
            iolog_stderr:      raw.get("iolog_stderr")      .unwrap_or(false),
            iolog_ttyin:       raw.get("iolog_ttyin")       .unwrap_or(false),
            iolog_ttyout:      raw.get("iolog_ttyout")      .unwrap_or(false),
            login_class:       raw.get("login_class")       .ok(),
            nice:              raw.get("nice")              .ok(),
            noexec:            raw.get("noexec")            .unwrap_or(false),
            preserve_fds:      raw.get("preserve_fds")      .unwrap_or_else(|_| vec![]),
            preserve_groups:   raw.get("preserve_groups")   .unwrap_or(false),
            runas_groups:      raw.get("runas_groups")      .ok(),
            selinux_role:      raw.get("selinux_role")      .ok(),
            selinux_type:      raw.get("selinux_type")      .ok(),
            set_utmp:          raw.get("set_utmp")          .unwrap_or(false),
            sudoedit:          raw.get("sudoedit")          .unwrap_or(false),
            sudoedit_checkdir: raw.get("sudoedit_checkdir") .unwrap_or(true),
            sudoedit_follow:   raw.get("sudoedit_follow")   .unwrap_or(false),
            timeout:           raw.get("timeout")           .ok(),
            use_pty:           raw.get("use_pty")           .unwrap_or(false),
            utmp_user:         raw.get("utmp_user")         .ok(),

            raw,
        })
    }
}
