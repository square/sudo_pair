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
use std::os::unix::io::RawFd;
use std::path::PathBuf;

use libc::{self, gid_t, mode_t, uid_t};

/// Information about the command being run. These values are used by
/// sudo to set the execution environment when running a command and set
/// by the policy plugin.
#[derive(Debug)]
pub struct CommandInfo {
    /// The root directory to use when running the command.
    pub chroot: Option<String>,

    /// If specified, sudo will close all files descriptors with a value
    /// of number or higher.
    pub close_from: Option<u64>,

    /// Fully qualified path to the command to be executed.
    pub command: PathBuf,

    /// The current working directory to change to when executing the command.
    pub cwd: Option<PathBuf>,

    /// By default, sudo runs a command as the foreground process as long as
    /// sudo itself is running in the foreground. When exec_background is
    /// enabled and the command is being run in a pseudo-terminal (due to I/O
    /// logging or the use_pty setting), the command will be run as a
    /// background process. Attempts to read from the controlling terminal (or
    /// to change terminal settings) will result in the command being suspended
    /// with the SIGTTIN signal (or SIGTTOU in the case of terminal settings).
    /// If this happens when sudo is a foreground process, the command will be
    /// granted the controlling terminal and resumed in the foreground with no
    /// user intervention required. The advantage of initially running the
    /// command in the background is that sudo need not read from the terminal
    /// unless the command explicitly requests it. Otherwise, any terminal
    /// input must be passed to the command, whether it has required it or not
    /// (the kernel buffers terminals so it is not possible to tell whether the
    /// command really wants the input). This is different from historic sudo
    /// behavior or when the command is not being run in a pseudo-terminal.
    ///
    /// For this to work seamlessly, the operating system must support the
    /// automatic restarting of system calls. Unfortunately, not all operating
    /// systems do this by default, and even those that do may have bugs. For
    /// example, macOS fails to restart the tcgetattr() and tcsetattr() system
    /// calls (this is a bug in macOS). Furthermore, because this behavior
    /// depends on the command stopping with the SIGTTIN or SIGTTOU signals,
    /// programs that catch these signals and suspend themselves with a
    /// different signal (usually SIGTOP) will not be automatically
    /// foregrounded. Some versions of the linux su(1) command behave this way.
    /// Because of this, a plugin should not set exec_background unless it is
    /// explicitly enabled by the administrator and there should be a way to
    /// enabled or disable it on a per-command basis.
    ///
    /// This setting has no effect unless I/O logging is enabled or use_pty is
    /// enabled.
    pub exec_background: bool,

    /// If specified, sudo will use the fexecve(2) system call to execute
    /// the command instead of execve(2). The specified number must refer to an
    /// open file descriptor.
    pub exec_fd: Option<u64>,

    /// Set to true if the I/O logging plugins, if any, should compress the log
    /// data. This is a hint to the I/O logging plugin which may choose to
    /// ignore it.
    pub iolog_compress: bool,

    /// The group that will own newly created I/O log files and directories.
    /// This is a hint to the I/O logging plugin which may choose to ignore it.
    pub iolog_group: Option<String>,

    /// The file permission mode to use when creating I/O log files and
    /// directories. This is a hint to the I/O logging plugin which may choose
    /// to ignore it.
    pub iolog_mode: Option<String>,

    /// Fully qualified path to the file or directory in which I/O log is to be
    /// stored. This is a hint to the I/O logging plugin which may choose to
    /// ignore it. If no I/O logging plugin is loaded, this setting has no
    /// effect.
    pub iolog_path: Option<String>,

    /// Set to true if the I/O logging plugins, if any, should log the standard
    /// input if it is not connected to a terminal device. This is a hint to
    /// the I/O logging plugin which may choose to ignore it.
    pub iolog_stdin: bool,

    /// Set to true if the I/O logging plugins, if any, should log the standard
    /// output if it is not connected to a terminal device. This is a hint to
    /// the I/O logging plugin which may choose to ignore it.
    pub iolog_stdout: bool,

    /// Set to true if the I/O logging plugins, if any, should log the standard
    /// error if it is not connected to a terminal device. This is a hint to
    /// the I/O logging plugin which may choose to ignore it.
    pub iolog_stderr: bool,

    /// Set to true if the I/O logging plugins, if any, should log all terminal
    /// input. This only includes input typed by the user and not from a pipe
    /// or redirected from a file. This is a hint to the I/O logging plugin
    /// which may choose to ignore it.
    pub iolog_ttyin: bool,

    /// Set to true if the I/O logging plugins, if any, should log all terminal
    /// output. This only includes output to the screen, not output to a pipe
    /// or file. This is a hint to the I/O logging plugin which may choose to
    /// ignore it.
    pub iolog_ttyout: bool,

    /// The user that will own newly created I/O log files and directories.
    /// This is a hint to the I/O logging plugin which may choose to ignore it.
    pub iolog_user: Option<String>,

    /// BSD login class to use when setting resource limits and nice value
    /// (optional). This option is only set on systems that support login
    /// classes.
    pub login_class: Option<String>,

    /// Nice value (priority) to use when executing the command. The nice
    /// value, if specified, overrides the priority associated with the
    /// login_class on BSD systems.
    pub nice: Option<u64>,

    /// If set, prevent the command from executing other programs.
    pub noexec: bool,

    /// A comma-separated list of file descriptors that should be preserved,
    /// regardless of the value of the closefrom setting. Only available
    /// starting with API version 1.5.
    pub preserve_fds: Vec<RawFd>,

    /// If set, sudo will preserve the user's group vector instead of
    /// initializing the group vector based on runas_user.
    pub preserve_groups: bool,

    /// Effective group-ID to run the command as. If not specified, the value
    /// of runas_gid is used.
    pub runas_egid: gid_t,

    /// Effective user-ID to run the command as. If not specified, the value of
    /// runas_uid is used.
    pub runas_euid: uid_t,

    /// Group-ID to run the command as.
    pub runas_gid: gid_t,

    /// The supplementary group vector to use for the command in the form of a
    /// comma-separated list of group-IDs. If preserve_groups is set, this
    /// option is ignored.
    pub runas_groups: Option<Vec<gid_t>>,

    /// User-ID to run the command as.
    pub runas_uid: uid_t,

    /// SELinux role to use when executing the command.
    pub selinux_role: Option<String>,

    /// SELinux type to use when executing the command.
    pub selinux_type: Option<String>,

    /// Create a utmp (or utmpx) entry when a pseudo-terminal is allocated. By
    /// default, the new entry will be a copy of the user's existing utmp entry
    /// (if any), with the tty, time, type and pid fields updated.
    pub set_utmp: bool,

    /// Set to true when in sudoedit mode. The plugin may enable sudoedit mode
    /// even if sudo was not invoked as sudoedit. This allows the plugin to
    /// perform command substitution and transparently enable sudoedit when the
    /// user attempts to run an editor.
    pub sudoedit: bool,

    /// Set to false to disable directory writability checks in sudoedit. By
    /// default, sudoedit 1.8.16 and higher will check all directory components
    /// of the path to be edited for writability by the invoking user. Symbolic
    /// links will not be followed in writable directories and sudoedit will
    /// refuse to edit a file located in a writable directory. These
    /// restrictions are not enforced when sudoedit is run by root. The
    /// sudoedit_follow option can be set to false to disable this check. Only
    /// available starting with API version 1.8.
    pub sudoedit_checkdir: bool,

    /// Set to true to allow sudoedit to edit files that are symbolic links. By
    /// default, sudoedit 1.8.15 and higher will refuse to open a symbolic
    /// link. The sudoedit_follow option can be used to restore the older
    /// behavior and allow sudoedit to open symbolic links. Only available
    /// starting with API version 1.8.
    pub sudoedit_follow: bool,

    /// Command timeout. If non-zero then when the timeout expires the command
    /// will be killed.
    pub timeout: Option<u64>,

    /// The file creation mask to use when executing the command. This value
    /// may be overridden by PAM or login.conf on some systems unless the
    /// umask_override option is also set.
    pub umask: mode_t,

    /// Force the value specified by the umask option to override any umask set
    /// by PAM or login.conf.
    pub umask_override: Option<bool>,

    /// Allocate a pseudo-terminal to run the command in, regardless of whether
    /// or not I/O logging is in use. By default, sudo will only run the
    /// command in a pseudo-terminal when an I/O log plugin is loaded.
    pub use_pty: bool,

    /// User name to use when constructing a new utmp (or utmpx) entry when
    /// set_utmp is enabled. This option can be used to set the user field in
    /// the utmp entry to the user the command runs as rather than the invoking
    /// user. If not set, sudo will base the new entry on the invoking user's
    /// existing entry.
    pub utmp_user: Option<String>,

    /// The raw underlying [`OptionMap`](OptionMap) to retrieve additional
    /// values that may not have been known at the time of the authorship of
    /// this file.
    pub raw: OptionMap,
}

impl TryFrom<OptionMap> for CommandInfo {
    type Error = Error;

    fn try_from(value: OptionMap) -> Result<Self> {
        let runas_gid = value.get("runas_gid")
            .unwrap_or_else(|_| unsafe { libc::getegid() });

        let runas_uid = value.get("runas_uid")
            .unwrap_or_else(|_| unsafe { libc::geteuid() });

        Ok(Self {
            // in the event that the `-V` flag is passed to `sudo`,
            // there's no command
            command:       value.get("command").unwrap_or_default(),
            runas_gid,
            runas_uid,
            runas_egid:    value.get("runas_egid").unwrap_or(runas_gid),
            runas_euid:    value.get("runas_euid").unwrap_or(runas_uid),
            umask:         value.get("umask").unwrap_or(0o7777),

            chroot:            value.get("chroot")            .ok(),
            close_from:        value.get("closefrom")         .ok(),
            cwd:               value.get("cwd")               .ok(),
            exec_background:   value.get("exec_background")   .unwrap_or(false),
            exec_fd:           value.get("execfd")            .ok(),
            iolog_compress:    value.get("iolog_compress")    .unwrap_or(false),
            iolog_group:       value.get("iolog_group")       .ok(),
            iolog_mode:        value.get("iolog_mode")        .ok(),
            iolog_path:        value.get("iolog_path")        .ok(),
            iolog_stdin:       value.get("iolog_stdin")       .unwrap_or(false),
            iolog_stdout:      value.get("iolog_stdout")      .unwrap_or(false),
            iolog_stderr:      value.get("iolog_stderr")      .unwrap_or(false),
            iolog_ttyin:       value.get("iolog_ttyin")       .unwrap_or(false),
            iolog_ttyout:      value.get("iolog_ttyout")      .unwrap_or(false),
            iolog_user:        value.get("iolog_user")        .ok(),
            login_class:       value.get("login_class")       .ok(),
            nice:              value.get("nice")              .ok(),
            noexec:            value.get("noexec")            .unwrap_or(false),
            preserve_fds:      value.get("preserve_fds")      .unwrap_or_default(),
            preserve_groups:   value.get("preserve_groups")   .unwrap_or(false),
            runas_groups:      value.get("runas_groups")      .ok(),
            selinux_role:      value.get("selinux_role")      .ok(),
            selinux_type:      value.get("selinux_type")      .ok(),
            set_utmp:          value.get("set_utmp")          .unwrap_or(false),
            sudoedit:          value.get("sudoedit")          .unwrap_or(false),
            sudoedit_checkdir: value.get("sudoedit_checkdir") .unwrap_or(true),
            sudoedit_follow:   value.get("sudoedit_follow")   .unwrap_or(false),
            timeout:           value.get("timeout")           .ok(),
            umask_override:    value.get("umask_override")    .ok(),
            use_pty:           value.get("use_pty")           .unwrap_or(false),
            utmp_user:         value.get("utmp_user")         .ok(),

            raw: value,
        })
    }
}
