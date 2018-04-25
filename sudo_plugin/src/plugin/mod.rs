//! Utilities for wrapping sudo plugins and the values they're
//! configured with.

mod option_map;
mod command_info;
mod settings;
mod user_info;

use super::errors::*;
use super::version::Version;

pub use self::option_map::OptionMap;

use self::command_info::CommandInfo;
use self::settings::Settings;
use self::user_info::UserInfo;

use sudo_plugin_sys;

use std::ffi::{CString, CStr};
use std::slice;

use libc::{c_char, c_int, c_uint};

/// An implementation of a sudo plugin, initialized and parsed from the
/// values passed to the underlying `open` callback.
#[allow(missing_debug_implementations)]
pub struct Plugin {
    /// The plugin API version supported by the invoked `sudo` command.
    pub version: Version,

    /// The command being executed, in the same form as would be passed
    /// to the `execve(2)` system call.
    pub command: Vec<CString>,

    /// A map of user-supplied sudo settings. These settings correspond
    /// to flags the user specified when running sudo. As such, they
    /// will only be present when the corresponding flag has been specified
    /// on the command line.
    pub settings: Settings,

    /// A map of information about the user running the command.
    pub user_info: UserInfo,

    /// A map of information about the command being run.
    pub command_info: CommandInfo,

    /// A map of the user's environment variables.
    pub user_env: OptionMap,

    /// A map of options provided to the plugin after the its path in
    /// sudo.conf.
    ///
    /// Settings that aren't of the form `key=value` will have a key
    /// in the map whose value is the same as the key, similar to how
    /// HTML handles valueless attributes (e.g., `disabled` will become
    /// `plugin_options["disabled"] => "disabled"`).
    pub plugin_options: OptionMap,

    _conversation: sudo_plugin_sys::sudo_conv_t,
    printf:        sudo_plugin_sys::sudo_printf_t,
}

impl Plugin {
    /// Initializes a `Plugin` from the arguments provided to the
    /// underlying C `open` callback function. Verifies the API version
    /// advertised by the underlying `sudo` is supported by this library,
    /// parses all provided options, and wires up communication
    /// facilities.
    ///
    /// Returns an error if there was a problem initializing the plugin.
    #[cfg_attr(feature="cargo-clippy", allow(cast_sign_loss))]
    #[cfg_attr(feature="cargo-clippy", allow(too_many_arguments))]
    pub unsafe fn new(
        version:        c_uint,
        argc:           c_int,
        argv:           *const *const c_char,
        conversation:   sudo_plugin_sys::sudo_conv_t,
        plugin_printf:  sudo_plugin_sys::sudo_printf_t,
        settings:       *const *const c_char,
        user_info:      *const *const c_char,
        command_info:   *const *const c_char,
        user_env:       *const *const c_char,
        plugin_options: *const *const c_char,
    ) -> Result<Self> {
        let version = Version::from(version).check()?;

        // verify we've been given needed callbacks; we actually store the
        // Option-wrapped variants (instead of unwrapping them) because
        // those are the types the `sudo_plugin_sys` crate exports
        let _ = plugin_printf.ok_or(ErrorKind::Uninitialized)?;
        let _ = conversation .ok_or(ErrorKind::Uninitialized)?;

        // parse the argv into the command being run
        let mut argv    = slice::from_raw_parts(argv, argc as usize).to_vec();
        let     command = argv
            .drain(..)
            .map(|ptr| CStr::from_ptr(ptr).to_owned())
            .collect();

        let plugin = Self {
            version,
            command,

            settings:       OptionMap::new(settings)    .and_then(Settings::new)?,
            user_info:      OptionMap::new(user_info)   .and_then(UserInfo::new)?,
            command_info:   OptionMap::new(command_info).and_then(CommandInfo::new)?,
            user_env:       OptionMap::new(user_env)?,
            plugin_options: OptionMap::new(plugin_options)?,

            _conversation: conversation,
            printf:        plugin_printf,
        };

        Ok(plugin)
    }

    ///
    /// As best as can be reconstructed, what was actually typed at the
    /// shell in order to launch this invocation of sudo.
    ///
    pub fn invocation(&self) -> Vec<u8> {
        let mut sudo    = self.settings.progname.as_bytes().to_vec();
        let     flags   = self.settings.flags();
        let     command = self.command_info.command.as_bytes();

        if !flags.is_empty() {
            sudo.push(b' ');
            sudo.extend_from_slice(&flags.join(&b' ')[..]);
        }

        if !command.is_empty() {
            sudo.push(b' ');
            sudo.extend_from_slice(&command);
        }

        sudo
    }

    /// Prints an informational message (which must not contain interior
    /// NUL bytes) to the plugin's `printf` facility.
    pub fn print_info<T: Into<Vec<u8>>>(&self, message: T) -> Result<c_int> {
        self.print(sudo_plugin_sys::SUDO_CONV_INFO_MSG, message)
    }

    /// Prints an error message (which must not contain interior NUL
    /// bytes) to the plugin's `printf` facility.
    pub fn print_error<T: Into<Vec<u8>>>(&self, message: T) -> Result<c_int> {
        self.print(sudo_plugin_sys::SUDO_CONV_ERROR_MSG, message)
    }

    /// Prints a message (which must not contain interior NUL bytes) to
    /// the plugin's `printf` facility using the requested severity
    /// level.
    fn print<T: Into<Vec<u8>>>(&self, level: c_uint, message: T) -> Result<c_int> {
        unsafe { Self::printf(self.printf, level, message) }
    }

    /// Prints a message (which must not contain interior NUL bytes) to
    /// the plugin's `printf` facility using the requested flags. This
    /// is provided as a static function in order to facilitate printing
    /// error messages before the plugin is fully initialized (for
    /// example, in the event of an initialization failure).
    pub unsafe fn printf<T: Into<Vec<u8>>>(
        printf:  sudo_plugin_sys::sudo_printf_t,
        flags:   c_uint,
        message: T,
    ) -> Result<c_int> {
        // TODO: level should be bitflags
        let printf  = printf.ok_or(ErrorKind::Uninitialized)?;
        let cstring = CString::new(message.into())
            .chain_err(|| ErrorKind::IoError(IoFacility::PluginPrintf))?;

        #[cfg_attr(feature="cargo-clippy", allow(cast_possible_wrap))]
        let ret = (printf)(flags as c_int, cstring.as_ptr());

        if ret == -1 {
            bail!(ErrorKind::IoError(IoFacility::PluginPrintf))
        }

        Ok(ret)
    }
}
