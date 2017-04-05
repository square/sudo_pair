//! sudo IO-plugin to require a live human pair.
//!
//! TODO: explain

#![deny(warnings)]

#![warn(fat_ptr_transmutes)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]

// this entire crate is practically unsafe code
#![allow(unsafe_code)]

// UnixStream requires the drop_types_in_cost feature
#![allow(unstable_features)]
#![feature(drop_types_in_const)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(clippy))]
#![cfg_attr(feature="clippy", warn(clippy_pedantic))]

extern crate libc;
extern crate unix_socket;

mod result;
mod session;
mod socket;
mod sudo;

use result::{Result, Error, SettingKind};
use session::{Session, Options};

use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::ffi::{CStr, CString};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::str;

use libc::{c_char, c_int, c_uint, sighandler_t, mode_t, pid_t, uid_t, gid_t};

const MSG_VERSION_MISMATCH: &'static [u8]
    = b"sudo: WARNING: API version %x, sudo_pair expects %x\n\0";

const MSG_PIPE_DISALLOWED: &'static [u8]
    = b"sudo: sudo_pair prohibits redirection of stdin, stdout, and stderr\n\0";

const MSG_SESSION_ENDED: &'static [u8]
    = b"\nsudo: sudo_pair session terminated\n\0";

const MSG_ERROR: &'static [u8]
    = b"sudo: %s\n\0";

static mut SUDO_PAIR_SESSION: Option<Session> = None;

static mut SUDO_CONVERSATION: Option<sudo::sudo_conv_t>   = None;
static mut SUDO_PRINTF:       Option<sudo::sudo_printf_t> = None;

/// The exported plugin function that hooks into sudo.
#[no_mangle]
pub static SUDO_PAIR_PLUGIN: sudo::io_plugin = sudo::io_plugin {
    type_:            sudo::SUDO_IO_PLUGIN,
    version:          sudo::SUDO_API_VERSION,
    open:             Some(sudo_pair_open),
    close:            Some(sudo_pair_close),
    show_version:     None,
    log_ttyin:        None,
    log_ttyout:       Some(sudo_pair_log_ttyout),
    log_stdin:        Some(sudo_pair_log_disabled),
    log_stdout:       Some(sudo_pair_log_disabled),
    log_stderr:       Some(sudo_pair_log_disabled),
    register_hooks:   None,
    deregister_hooks: None,
};

macro_rules! sudo_printf {
    ( $message:expr , $( $arg:expr ),* ) => ({
        let _ = SUDO_PRINTF.map(|printf| {
            let _ = printf(
                sudo::SUDO_CONV_INFO_MSG,
                $message.as_ptr() as _,
                $( $arg ),*
            );
        });
    });

    ( $message:expr ) => ( sudo_printf!{ $message, } );
}

unsafe extern "C" fn sudo_pair_open(
    version:            c_uint,
    conversation:       sudo::sudo_conv_t,
    sudo_printf:        sudo::sudo_printf_t,
    settings_ptr:       *const *mut c_char,
    user_info_ptr:      *const *mut c_char,
    command_info_ptr:   *const *mut c_char,
    _argc:              c_int,
    _argv:              *const *mut c_char,
    user_env_ptr:       *const *mut c_char,
    plugin_options_ptr: *const *mut c_char
) -> c_int {
    // set the global-scope conversation and s functions
    SUDO_CONVERSATION = Some(conversation);
    SUDO_PRINTF       = Some(sudo_printf);

    // warn if we're using a potentially incompatible plugin version
    if version != sudo::SUDO_API_VERSION {
        let _ = sudo_printf!(
            MSG_VERSION_MISMATCH,
            version,
            sudo::SUDO_API_VERSION
        );
    }

    SUDO_PAIR_SESSION = match sudo_pair_open_real(
        settings_ptr,
        user_info_ptr,
        command_info_ptr,
        user_env_ptr,
        plugin_options_ptr,
    ) {
        Ok(sess) => Some(sess),
        Err(e)   => {
            sudo_printf!(
                MSG_ERROR,
                CString::new(e.description()).unwrap()
            );

            return -1
        }
    };

    return 1;
}

unsafe fn sudo_pair_open_real(
    settings_ptr:       *const *mut c_char,
    user_info_ptr:      *const *mut c_char,
    command_info_ptr:   *const *mut c_char,
    user_env_ptr:       *const *mut c_char,
    plugin_options_ptr: *const *mut c_char
) -> Result<Session> {
    // TODO: errors
    let settings       = parse_option_vector(settings_ptr as _);
    let user_info      = parse_option_vector(user_info_ptr as _);
    let command_info   = parse_option_vector(command_info_ptr as _);
    let user_env       = parse_option_vector(user_env_ptr as _);
    let plugin_options = parse_option_vector(plugin_options_ptr as _);

    let runas_user = settings.get("runas_user")
        .ok_or(Error::MissingSetting(SettingKind::Settings, "runas_user"))?;

    let pid = user_info.get("pid")
       .ok_or(Error::MissingSetting(SettingKind::UserInfo, "pid"))?
       .parse::<pid_t>()?;

    let uid = user_info.get("uid")
       .ok_or(Error::MissingSetting(SettingKind::UserInfo, "uid"))?
       .parse()?;

    let gids = user_info.get("groups")
        .ok_or(Error::MissingSetting(SettingKind::UserInfo, "groups"))?
        .split(',')
        .filter_map(|gid| gid.parse().ok())
        .collect();

    let cwd = user_info.get("cwd")
        .ok_or(Error::MissingSetting(SettingKind::UserInfo, "cwd"))?;

    let host = user_info.get("host")
        .ok_or(Error::MissingSetting(SettingKind::UserInfo, "host"))?;

    let command = command_info.get("command")
        .ok_or(Error::MissingSetting(SettingKind::CommandInfo, "command"))?;

    let runas_uid = command_info.get("runas_uid")
        .ok_or(Error::MissingSetting(SettingKind::CommandInfo, "runas_uid"))?
        .parse()?;

    let runas_gid = command_info.get("runas_gid")
        .ok_or(Error::MissingSetting(SettingKind::CommandInfo, "runas_gid"))?
        .parse()?;

    let user = user_env.get("SUDO_USER")
        .ok_or(Error::MissingSetting(SettingKind::UserEnv, "SUDO_USER"))?;

    let options = PluginOptions::from(plugin_options);

    // force the session to be exempt if we're running the approval
    // command
    let exempt = options.binary_path == PathBuf::from(&command);

    if !exempt {
        println!("\
Running this command requires another user to approve and watch your \
session. Please have another user run

\tssh {} 'sudo -u {} {} {}'\n",
            host,
            runas_user,
            options.binary_path.to_string_lossy(),
            pid,
        );
    }

    // temporarily install a SIGINT handler while we block on accept()
    // TODO: handle errors
    let sigint = signal(libc::SIGINT, ctrl_c as _).unwrap();

    // TODO: error messaging
    let mut session = Session::new(
        options.socket_dir.join(pid.to_string()).with_extension("sock"),
        uid,
        gids,
        Options {
            socket_uid:    options.socket_uid.unwrap_or(runas_uid),
            socket_gid:    options.socket_gid.unwrap_or(runas_gid),
            socket_mode:   options.socket_mode,
            gids_enforced: options.gids_enforced,
            gids_exempted: options.gids_exempted,
            exempt:        exempt,
        },
    )?;

    if session.is_exempt() {
        return Ok(session);
    }

    // TODO: handle return value
    // \t<{}@{}:{} $ > sudo -u {} {}
    let _ = session.write_all(
        format!("\
User {} is attempting to run

\t<\x1b[1;34m{}\x1b[0m@\x1b[1;32m{}\x1b[0m:\x1b[01;34m{}\x1b[0m\x1b[1;32m $\x1b[0m > sudo -u {} {}

If you approve, you will see the live session through this terminal. To \
immediately abort the interactive session (and kill the approved sudo \
command), press ESC.

Please note: if you abandon this session, it will kill the approved sudo \
command.

Approve? y/n [n]: ",
            user,
            user,
            host,
            cwd,
            runas_user,
            command,
        ).as_bytes()
    );

    let mut response = [0];

    // read one byte from the socket
    session.read_exact(&mut response)?;

    // echo back out the response, since it's noecho, raw on the client
    let _ = session.write_all(&response[..]);
    let _ = session.write_all(b"\n");

    // restore the original SIGINT handler
    // TODO: handle errors
    let _ = signal(libc::SIGINT, sigint).unwrap();

    // if those two bytes were a "yes", we're authorized to
    // open a session; otherwise we've been declined
    match &response {
        b"y" => Ok(session),
        b"Y" => Ok(session),
        _    => Err(Error::Unauthorized),
    }
}

unsafe extern "C" fn sudo_pair_close(_exit_status: c_int, _error: c_int) {
    // TODO: exit status
    SUDO_PAIR_SESSION = None
}

unsafe extern "C" fn sudo_pair_log_ttyout(
    buf: *const c_char,
    len: c_uint
) -> c_int {
    let mut sess = match SUDO_PAIR_SESSION.as_mut() {
        Some(sess) => sess,
        None       => return 0, // no session means we didn't initialize
    };

    match sess.write_all(std::slice::from_raw_parts(buf as _, len as _)) {
        Ok(_)  => return 1,
        Err(_) => {
            sudo_printf!(MSG_SESSION_ENDED);

            // socket is closed, kill the command;
            // TODO: return 0 broken, segfaults?
            return -1
        },
    };
}

unsafe extern "C" fn sudo_pair_log_disabled(
    _buf: *const c_char,
    _len: c_uint
) -> c_int {
    sudo_printf!(MSG_PIPE_DISALLOWED);

    return -1;
}

unsafe fn parse_option_vector(
    mut ptr: *const *const c_char,
) -> HashMap<String, String> {
    let mut hash = HashMap::new();

    while !(*ptr).is_null() {
        let cstr      = CStr::from_ptr(*ptr).to_string_lossy();
        let mut pair  = cstr.split('=');
        let key       = pair.next().unwrap().to_string();
        let value     = pair.collect::<String>();

        let _ = hash.insert(key, value);

        ptr = ptr.offset(1);
    }

    return hash;
}

fn parse_delimited_string<F, T>(
    string: &str,
    delimiter: char,
    parser: F,
) -> HashSet<T>
    where F: FnMut(&str) -> T, T: Eq + std::hash::Hash
         {
    string.split(delimiter).map(parser).collect()
}

unsafe fn signal(signum: c_int, handler: sighandler_t) -> io::Result<sighandler_t> {
    match libc::signal(signum, handler) {
        libc::SIG_ERR => Err(io::Error::last_os_error()),
        handler       => Ok(handler),
    }
}

// TODO: There's not much we can do in a signal handler, but
// _exit(3) is safe (exit(3) isn't!). Ideally we'd close(2) the
// socket blocking on accept(2) instead of exiting directly, but
// that's more complicated than it's worth for now.
unsafe extern "C" fn ctrl_c(_sig: c_int) {
    // sudo normally exits with exit code 1 if you Ctrl-C during
    // password entry, so we retain that convention
    libc::_exit(1);
}

struct PluginOptions {
    binary_path: PathBuf,
    socket_dir:  PathBuf,
    socket_uid:  Option<uid_t>,
    socket_gid:  Option<gid_t>,
    socket_mode: mode_t,

    gids_enforced: HashSet<gid_t>,
    gids_exempted: HashSet<gid_t>,
}

impl Default for PluginOptions {
    fn default() -> Self {
        PluginOptions {
            binary_path:   PathBuf::from("/usr/bin/sudo_pair_approve"),
            socket_dir:    PathBuf::from("/var/run/sudo_pair"),
            socket_uid:    None,
            socket_gid:    None,
            socket_mode:   0o700,
            gids_enforced: HashSet::new(),
            gids_exempted: HashSet::new(),
        }
    }
}

impl From<HashMap<String, String>> for PluginOptions {
    fn from(map: HashMap<String, String>) -> Self {
        let mut options = Self::default();

        for (key, value) in &map {
            match &key[..] {
                "BinaryPath"   => options.binary_path   = PathBuf::from(value),
                "SocketDir"    => options.socket_dir    = PathBuf::from(value),
                "SocketUid"    => options.socket_uid    = Some(value.parse().unwrap()),
                "SocketGid"    => options.socket_gid    = Some(value.parse().unwrap()),
                "SocketMode"   => options.socket_mode   = mode_t::from_str_radix(value, 8).unwrap(),
                "GidsEnforced" => options.gids_enforced = parse_delimited_string(&value, ',', |s| s.parse().unwrap()),
                "GidsExempted" => options.gids_exempted = parse_delimited_string(&value, ',', |s| s.parse().unwrap()),
                _              => (), // TODO: warn
            }
        }

        options
    }
}
