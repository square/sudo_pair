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
use super::parsing::*;

use libc::{c_char, gid_t, pid_t, uid_t};

#[derive(Debug)]
pub struct UserInfo {
    pub cols:   u64,
    pub cwd:    String,
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
    pub tty:    Option<String>,
    pub uid:    uid_t,
    pub user:   String,

    pub raw: RawOptions,
}

impl UserInfo {
   pub fn new(ptr: *const *const c_char) -> Result<Self> {
        let raw = unsafe {
            RawOptions::new(ptr)
        }?;

        Ok(UserInfo {
            cwd:    raw.get_parsed("cwd")?,
            egid:   raw.get_parsed("egid")?,
            euid:   raw.get_parsed("euid")?,
            gid:    raw.get_parsed("gid")?,
            groups: raw.get_parsed("groups")?,
            host:   raw.get_parsed("host")?,
            pgid:   raw.get_parsed("pgid")?,
            pid:    raw.get_parsed("pid")?,
            ppid:   raw.get_parsed("ppid")?,
            uid:    raw.get_parsed("uid")?,
            user:   raw.get_parsed("user")?,

            cols:   raw.get_parsed("cols")  .unwrap_or(80),
            lines:  raw.get_parsed("lines") .unwrap_or(24),
            sid:    raw.get_parsed("sid")   .unwrap_or(0),
            tcpgid: raw.get_parsed("tcpgid").unwrap_or(-1),
            tty:    raw.get_parsed("tty")   .ok(),

            raw:    raw,
        })
    }
}
