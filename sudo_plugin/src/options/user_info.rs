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

use crate::errors::{Result, Error};
use crate::options::OptionMap;

use std::convert::TryFrom;
use std::path::PathBuf;

use libc::{gid_t, pid_t, uid_t};

/// A vector of information about the user running the command.
#[derive(Debug)]
pub struct UserInfo {
    /// The number of columns the user's terminal supports. If there is no
    /// terminal device available, a default value of 80 is used.
    pub cols: u64,

    /// The user's current working directory.
    pub cwd: PathBuf,

    /// The effective group-ID of the user invoking sudo.
    pub egid: gid_t,

    /// The effective user-ID of the user invoking sudo.
    pub euid: uid_t,

    /// The real group-ID of the user invoking sudo.
    pub gid: gid_t,

    /// The user's supplementary group list formatted as a string of
    /// comma-separated group-IDs.
    pub groups: Vec<gid_t>,

    /// The local machine's hostname as returned by the gethostname(2) system
    /// call.
    pub host: String,

    /// The number of lines the user's terminal supports. If there is no
    /// terminal device available, a default value of 24 is used.
    pub lines: u64,

    /// The ID of the process group that the running sudo process is a member
    /// of. Only available starting with API version 1.2.
    pub pgid: pid_t,

    /// The process ID of the running sudo process. Only available starting
    /// with API version 1.2.
    pub pid: pid_t,

    /// The parent process ID of the running sudo process. Only available
    /// starting with API version 1.2.
    pub ppid: pid_t,

    /// The session ID of the running sudo process or 0 if sudo is not part of
    /// a POSIX job control session. Only available starting with API version
    /// 1.2.
    pub sid: pid_t,

    /// The ID of the foreground process group associated with the terminal
    /// device associated with the sudo process or -1 if there is no terminal
    /// present. Only available starting with API version 1.2.
    pub tcpgid: pid_t,

    /// The path to the user's terminal device. If the user has no terminal
    /// device associated with the session, the value will be empty, as in
    /// “tty=”.
    pub tty: Option<PathBuf>,

    /// The real user-ID of the user invoking sudo.
    pub uid: uid_t,

    /// The invoking user's file creation mask. Only available starting with
    /// API version 1.10.
    pub umask: Option<String>,

    /// The name of the user invoking sudo.
    pub user: String,

    /// The raw underlying [`OptionMap`](OptionMap) to retrieve additional
    /// values that may not have been known at the time of the authorship of
    /// this file.
    pub raw: OptionMap,
}

impl TryFrom<OptionMap> for UserInfo {
    type Error = Error;

    fn try_from(value: OptionMap) -> Result<Self> {
        Ok(Self {
            cwd:    value.get("cwd")?,
            egid:   value.get("egid")?,
            euid:   value.get("euid")?,
            gid:    value.get("gid")?,
            groups: value.get("groups")?,
            host:   value.get("host")?,
            pgid:   value.get("pgid")?,
            pid:    value.get("pid")?,
            ppid:   value.get("ppid")?,
            uid:    value.get("uid")?,
            user:   value.get("user")?,

            umask:  value.get("umask") .ok(),
            cols:   value.get("cols")  .unwrap_or(80),
            lines:  value.get("lines") .unwrap_or(24),
            sid:    value.get("sid")   .unwrap_or(0),
            tcpgid: value.get("tcpgid").unwrap_or(-1),
            tty:    value.get("tty")   .ok(),

            raw: value,
        })
    }
}
