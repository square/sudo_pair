use super::super::errors::*;
use super::option_map::*;

use libc::{gid_t, pid_t, uid_t};

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

    pub raw: OptionMap,
}

impl UserInfo {
    pub fn new(raw: OptionMap) -> Result<Self> {
        Ok(Self {
            cwd:    raw.get("cwd")?,
            egid:   raw.get("egid")?,
            euid:   raw.get("euid")?,
            gid:    raw.get("gid")?,
            groups: raw.get("groups")?,
            host:   raw.get("host")?,
            pgid:   raw.get("pgid")?,
            pid:    raw.get("pid")?,
            ppid:   raw.get("ppid")?,
            uid:    raw.get("uid")?,
            user:   raw.get("user")?,

            cols:   raw.get("cols")  .unwrap_or(80),
            lines:  raw.get("lines") .unwrap_or(24),
            sid:    raw.get("sid")   .unwrap_or(0),
            tcpgid: raw.get("tcpgid").unwrap_or(-1),
            tty:    raw.get("tty")   .ok(),

            raw,
        })
    }
}
