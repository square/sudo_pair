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

use crate::errors::*;
use super::option_map::*;

use std::path::PathBuf;

use libc::{gid_t, pid_t, uid_t};

#[derive(Debug)]
pub struct UserInfo {
    pub cols:   u64,
    pub cwd:    PathBuf,
    pub egid:   gid_t,
    pub euid:   uid_t,
    pub gid:    gid_t,
    pub groups: Vec<gid_t>,
    pub host:   String,
    pub lines:  u64,
    pub pgid:   pid_t,
    pub pid:    pid_t,
    pub ppid:   pid_t,
    pub sid:    pid_t,
    pub tcpgid: pid_t,
    pub tty:    Option<PathBuf>,
    pub uid:    uid_t,
    pub umask:  Option<String>,
    pub user:   String,

    pub raw: OptionMap,
}

impl UserInfo {
    pub fn try_from(value: OptionMap) -> Result<Self> {
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
