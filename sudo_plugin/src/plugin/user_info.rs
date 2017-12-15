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
use std::ffi::CString;

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

    pub raw: HashMap<CString, CString>,
}

impl UserInfo {
   pub fn new(ptr: *const *const c_char) -> Result<Self> {
        let raw = unsafe {
            parsing::parse_options(ptr)
        }?;

        Ok(UserInfo {
            cwd:    parsing::parse_raw(&raw, b"cwd\0",    parsing::parse)?,
            egid:   parsing::parse_raw(&raw, b"egid\0",   parsing::parse)?,
            euid:   parsing::parse_raw(&raw, b"euid\0",   parsing::parse)?,
            gid:    parsing::parse_raw(&raw, b"gid\0",    parsing::parse)?,
            groups: parsing::parse_raw(&raw, b"groups\0", parsing::parse_gids)?,
            host:   parsing::parse_raw(&raw, b"host\0",   parsing::parse)?,
            pgid:   parsing::parse_raw(&raw, b"pgid\0",   parsing::parse)?,
            pid:    parsing::parse_raw(&raw, b"pid\0",    parsing::parse)?,
            ppid:   parsing::parse_raw(&raw, b"ppid\0",   parsing::parse)?,
            uid:    parsing::parse_raw(&raw, b"uid\0",    parsing::parse)?,
            user:   parsing::parse_raw(&raw, b"user\0",   parsing::parse)?,

            cols:   parsing::parse_raw(&raw, b"cols\0",   parsing::parse).unwrap_or(80),
            lines:  parsing::parse_raw(&raw, b"lines\0",  parsing::parse).unwrap_or(24),
            sid:    parsing::parse_raw(&raw, b"sid\0",    parsing::parse).unwrap_or(0),
            tcpgid: parsing::parse_raw(&raw, b"tcpgid\0", parsing::parse).unwrap_or(-1),
            tty:    parsing::parse_raw(&raw, b"tty\0",    parsing::parse).ok(),

            raw:    raw,
        })
    }
}
