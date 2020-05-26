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

//! This module implements the actual `sudo_plugin(8)` callbacks that
//! convert between C and Rust style and trampoline into the acutal
//! plugin code. It is not intended for direct end-user use.

// This is entirely called from `C` code. Rust `#[must_use]` attributes
// aren't going to affect anything.
#![allow(clippy::must_use_candidate)]

use crate::errors::SudoError;
use crate::output::PrintFacility;
use crate::plugin::{IoEnv, IoPlugin, IoState};
use crate::sys;

use std::os::raw;
use std::path::PathBuf;

/// Return codes understood by the `io_plugin.open` callback.
///
/// The interpretations of these values are badly-documented within the
/// [`sudo_plugin(8)` manpage][manpage] so the code was used to
/// understand their actual effects.
///
/// [manpage]: https://www.sudo.ws/man/1.8.30/sudo_plugin.man.html
/// [code]: https://github.com/sudo-project/sudo/blob/446ae3f507271c8a08f054c9291cb8804afe81d9/src/sudo.c#L1404
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum OpenStatus {
    /// The plugin was `open`ed successfully and may be used as normal.
    Ok = 1,

    /// The plugin should be unloaded for the duration of this `sudo`
    /// session. The `sudo` session may continue, but will not use any
    /// of the features of this plugin.
    Disable = 0,

    /// The `sudo` command is unauthorized and must be immediately
    /// terminated.
    Deny = -1,

    /// The `sudo` command was invoked incorrectly and will be
    /// terminated. Basic usage information will be presented to the
    /// user. The plugin may choose to emit its own usage information
    /// describing the problem.
    Usage = -2,
}

/// Return codes understood by the `io_plugin.log_*` family of callbacks.
///
/// The interpretations of these values are badly-documented within the
/// [`sudo_plugin(8)` manpage][manpage] so the code was used to
/// understand their actual effects.
///
/// [manpage]: https://www.sudo.ws/man/1.8.30/sudo_plugin.man.html
/// [code]: https://github.com/sudo-project/sudo/blob/446ae3f507271c8a08f054c9291cb8804afe81d9/src/sudo.c#L1404
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum LogStatus {
    /// The plugin logged the information successfully.
    Ok = 1,

    /// The plugin has determined that the `sudo` session should be
    /// terminated immediately.
    Deny = 0,

    /// The plugin no longer needs this callback. This callback will no
    /// longer be invoked by `sudo`, but the rest of the plugin's
    /// callbacks will function as normal.
    Disable = -1,
}

impl<T, E: SudoError> From<Result<T, E>> for OpenStatus {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(_)  => OpenStatus::Ok,
            Err(e) => e.into(),
        }
    }
}

impl<T, E: SudoError> From<Result<T, E>> for LogStatus {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(_)  => LogStatus::Ok,
            Err(e) => e.into(),
        }
    }
}

#[doc(hidden)]
pub unsafe extern "C" fn open<P: 'static + IoPlugin, S: IoState<P>>(
    version: raw::c_uint,
    conversation:       sys::sudo_conv_t,
    plugin_printf:      sys::sudo_printf_t,
    settings_ptr:       *const *mut raw::c_char,
    user_info_ptr:      *const *mut raw::c_char,
    command_info_ptr:   *const *mut raw::c_char,
    argc:               raw::c_int,
    argv:               *const *mut raw::c_char,
    user_env_ptr:       *const *mut raw::c_char,
    plugin_options_ptr: *const *mut raw::c_char,
) -> raw::c_int {
    let static_io_env    = S::io_env();
    let static_io_plugin = S::io_plugin();

    // create our own PrintFacility to log to in case IoEnv
    // initialization fails
    let (_, mut stderr) = PrintFacility::new(
        Some(P::NAME), plugin_printf
    );

    let io_env = IoEnv::new(
        P::NAME,
        P::VERSION,
        version,
        argc, argv,
        settings_ptr,
        user_info_ptr,
        command_info_ptr,
        user_env_ptr,
        plugin_options_ptr,
        plugin_printf,
        conversation,
    );

    let io_env = match io_env {
        Ok(v)   => v,
        Err(e)  => {
            let _ = stderr.write_error(&e);
            let e : P::Error = e.into();
            let v            = Into::<OpenStatus>::into(e);
            return v as _;
        }
    };

    // Long story short, the `IoEnv` needs to be moved into static
    // memory before we initialize the user-provided `IoPlugin`. But
    // doing so puts it into an `Option` so we have to get a reference
    // out of the option. I hate using `unwrap` but in this case we know
    // it exists since we literally just assigned to it.
    //
    // TODO: can we avoid this dance?
    let _      = static_io_env.replace(io_env);
    let io_env = static_io_env.as_ref().unwrap();

    // if the command is empty, to the best of my knowledge
    // we're being called with `-V` to report our version; in
    // this case there's no reason to fully invoke the plugin
    // through its `open` function
    //
    // TODO: find a canonical way to test for `-V`
    // TODO: this test should go into the IoEnv
    // TODO: maybe model this as an "error"?
    if io_env.command_info.command == PathBuf::default() {
        // even though we're avoiding instantiating the plugin fully,
        // we need to make sure it makes its way into static storage
        // before returning
        return sys::SUDO_PLUGIN_OPEN_SUCCESS;
    }

    let io_plugin = match P::open(io_env) {
        Ok(v)  => v,
        Err(e) => {
            let _ = stderr.write_error(&e);
            let v = Into::<OpenStatus>::into(e);
            return v as _;
        },
    };

    let _ = static_io_plugin.replace(io_plugin);

    sys::SUDO_PLUGIN_OPEN_SUCCESS
}

#[doc(hidden)]
pub unsafe extern "C" fn close<P: 'static + IoPlugin, S: IoState<P>>(
    exit_status: raw::c_int,
    error:       raw::c_int,
) {
    // `close` takes ownership of the plugin and doesn't return it, so
    // the plugin is dropped once `close` exits
    //
    // # SAFETY it's extremely important that this method be called
    // *before* `S::io_env()` is taken, because the plugin may hold a
    // reference to the static IoEnv memory
    let _ = S::io_plugin().take()
        .map(|p| p.close(exit_status, error));

    // it's unnecessary to actually drop env explicitly, but doing so
    // intent that it cease to exist
    drop(S::io_env().take());
}

#[doc(hidden)]
pub unsafe extern "C" fn show_version<P: 'static + IoPlugin, S: IoState<P>>(
    verbose: raw::c_int,
) -> raw::c_int {
    let env = match S::io_env().as_ref() {
        Some(env) => env,
        None      => return sys::SUDO_PLUGIN_OPEN_FAILURE,
    };

    P::show_version(env, verbose != 0);

    sys::SUDO_PLUGIN_OPEN_SUCCESS
}

#[doc(hidden)]
pub unsafe extern "C" fn log_ttyin<P: 'static + IoPlugin, S: IoState<P>>(
    buf: *const raw::c_char,
    len:        raw::c_uint,
) -> raw::c_int {
    let env = match S::io_env() {
        Some(e) => e,
        None    => return sys::SUDO_PLUGIN_LOG_ERROR,
    };

    let plugin = match S::io_plugin() {
        Some(e) => e,
        None    => return sys::SUDO_PLUGIN_LOG_ERROR,
    };

    if !env.command_info.iolog_ttyin && !P::IGNORE_IOLOG_HINTS {
        return sys::SUDO_PLUGIN_LOG_OK;
    }

    let slice = ::std::slice::from_raw_parts(
        buf as *const _,
        len as _,
    );

    Into::<LogStatus>::into(plugin.log_ttyin(slice).map_err(|err| {
        let _ = env.stderr().write_error(&err);
        err
    })) as _
}

#[doc(hidden)]
pub unsafe extern "C" fn log_ttyout<P: 'static + IoPlugin, S: IoState<P>>(
    buf: *const raw::c_char,
    len:        raw::c_uint,
) -> raw::c_int {
    let env = match S::io_env() {
        Some(e) => e,
        None    => return sys::SUDO_PLUGIN_LOG_ERROR,
    };

    let plugin = match S::io_plugin() {
        Some(e) => e,
        None    => return sys::SUDO_PLUGIN_LOG_ERROR,
    };

    if !env.command_info.iolog_ttyout && !P::IGNORE_IOLOG_HINTS {
        return sys::SUDO_PLUGIN_LOG_OK;
    }

    let slice = ::std::slice::from_raw_parts(
        buf as *const _,
        len as _,
    );

    Into::<LogStatus>::into(plugin.log_ttyout(slice).map_err(|err| {
        let _ = env.stderr().write_error(&err);
        err
    })) as _
}

#[doc(hidden)]
pub unsafe extern "C" fn log_stdin<P: 'static + IoPlugin, S: IoState<P>>(
    buf: *const raw::c_char,
    len:        raw::c_uint,
) -> raw::c_int {
    let env = match S::io_env() {
        Some(e) => e,
        None    => return sys::SUDO_PLUGIN_LOG_ERROR,
    };

    let plugin = match S::io_plugin() {
        Some(e) => e,
        None    => return sys::SUDO_PLUGIN_LOG_ERROR,
    };

    if !env.command_info.iolog_stdin && !P::IGNORE_IOLOG_HINTS {
        return sys::SUDO_PLUGIN_LOG_OK;
    }

    let slice = ::std::slice::from_raw_parts(
        buf as *const _,
        len as _,
    );

    Into::<LogStatus>::into(plugin.log_stdin(slice).map_err(|err| {
        let _ = env.stderr().write_error(&err);
        err
    })) as _
}

#[doc(hidden)]
pub unsafe extern "C" fn log_stdout<P: 'static + IoPlugin, S: IoState<P>>(
    buf: *const raw::c_char,
    len:        raw::c_uint,
) -> raw::c_int {
    let env = match S::io_env() {
        Some(e) => e,
        None    => return sys::SUDO_PLUGIN_LOG_ERROR,
    };

    let plugin = match S::io_plugin() {
        Some(e) => e,
        None    => return sys::SUDO_PLUGIN_LOG_ERROR,
    };

    if !env.command_info.iolog_stdout && !P::IGNORE_IOLOG_HINTS {
        return sys::SUDO_PLUGIN_LOG_OK;
    }

    let slice = ::std::slice::from_raw_parts(
        buf as *const _,
        len as _,
    );

    Into::<LogStatus>::into(plugin.log_stdout(slice).map_err(|err| {
        let _ = env.stderr().write_error(&err);
        err
    })) as _
}

#[doc(hidden)]
pub unsafe extern "C" fn log_stderr<P: 'static + IoPlugin, S: IoState<P>>(
    buf: *const raw::c_char,
    len:        raw::c_uint,
) -> raw::c_int {
    let env = match S::io_env() {
        Some(e) => e,
        None    => return sys::SUDO_PLUGIN_LOG_ERROR,
    };

    let plugin = match S::io_plugin() {
        Some(e) => e,
        None    => return sys::SUDO_PLUGIN_LOG_ERROR,
    };

    if !env.command_info.iolog_stderr && !P::IGNORE_IOLOG_HINTS {
        return sys::SUDO_PLUGIN_LOG_OK;
    }

    let slice = ::std::slice::from_raw_parts(
        buf as *const _,
        len as _,
    );

    Into::<LogStatus>::into(plugin.log_stderr(slice).map_err(|err| {
        let _ = env.stderr().write_error(&err);
        err
    })) as _
}
