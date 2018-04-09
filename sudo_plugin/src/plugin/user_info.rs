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

            raw,
        })
    }
}
