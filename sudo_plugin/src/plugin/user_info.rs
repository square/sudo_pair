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

            cols:   value.get("cols")  .unwrap_or(80),
            lines:  value.get("lines") .unwrap_or(24),
            sid:    value.get("sid")   .unwrap_or(0),
            tcpgid: value.get("tcpgid").unwrap_or(-1),
            tty:    value.get("tty")   .ok(),

            raw: value,
        })
    }
}
