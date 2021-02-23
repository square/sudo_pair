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

use crate::{errors::{Error, SudoError}, output::{ConversationFacility, PrintFacility}, plugin::IoEnv};
use crate::plugin::{IoPlugin, IoState};
use crate::sys;

use std::{os::raw, path::PathBuf};
use std::panic::{catch_unwind, UnwindSafe};

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

impl<T: Into<OpenStatus>> From<std::thread::Result<T>> for OpenStatus {
    fn from(result: std::thread::Result<T>) -> Self {
        match result {
            Ok(v)  => v.into(),
            Err(_) => Error::UncaughtPanic.into(),
        }
    }
}

impl<T: Into<LogStatus>> From<std::thread::Result<T>> for LogStatus {
    fn from(result: std::thread::Result<T>) -> Self {
        match result {
            Ok(v)  => v.into(),
            Err(_) => Error::UncaughtPanic.into(),
        }
    }
}

fn catch_unwind_open<T: std::fmt::Debug + Into<OpenStatus>, F: FnOnce() -> T + UnwindSafe>(f: F) -> i32 {
    Into::<OpenStatus>::into(catch_unwind(f)) as _
}

fn catch_unwind_log<T: Into<LogStatus>, F: FnOnce() -> T + UnwindSafe>(f: F) -> i32 {
    Into::<LogStatus>::into(catch_unwind(f)) as _
}

#[doc(hidden)]
pub unsafe extern "C" fn open<P: IoPlugin, S: IoState<P>>(
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
    catch_unwind_open(|| {
        // create our own PrintFacility to log to in case IoEnv
        // initialization fails
        let (_, mut stderr) = PrintFacility::new(
            Some(P::NAME), plugin_printf
        );

        let conv_f = ConversationFacility::new(conversation);

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
            conv_f,
        );

        let io_env = match io_env {
            Ok(v)   => v,
            Err(e)  => {
                let _ = stderr.write_error(&e);
                let e : P::Error = e.into();
                return Into::<OpenStatus>::into(e);
            }
        };

        S::init(io_env, |env| {
            // even though we're avoiding instantiating the plugin
            // fully, we need to make sure it makes its way into static
            // storage before returning, which is why we put this check
            // inside `S::init`
            if env.command_info.command == PathBuf::default() {
                return Err(OpenStatus::Ok);
            }

            P::open(env).map_err(|e| {
                let _ = stderr.write_error(&e);
                Into::<OpenStatus>::into(e)
            })
        })
    })
}

#[doc(hidden)]
pub unsafe extern "C" fn close<P: IoPlugin, S: IoState<P>>(
    exit_status: raw::c_int,
    error:       raw::c_int,
) {
    drop(catch_unwind(|| {
        S::drop(|plugin| plugin.close(exit_status, error));
    }));
}

#[doc(hidden)]
pub unsafe extern "C" fn show_version<P: IoPlugin, S: IoState<P>>(
    verbose: raw::c_int,
) -> raw::c_int {
    catch_unwind_open(|| {
        P::show_version(S::io_env(), verbose != 0);

        OpenStatus::Ok
    })
}

#[doc(hidden)]
pub unsafe extern "C" fn log_ttyin<P: IoPlugin, S: IoState<P>>(
    buf: *const raw::c_char,
    len:        raw::c_uint,
) -> raw::c_int {
    catch_unwind_log(|| {
        let env    = S::io_env();
        let plugin = S::io_plugin();

        if !env.command_info.iolog_ttyin && !P::IGNORE_IOLOG_HINTS {
            return Ok(());
        }

        let slice = ::std::slice::from_raw_parts(
            buf as *const _,
            len as _,
        );

        plugin.log_ttyin(slice).map_err(|err| {
            let _ = env.stderr().write_error(&err);
            err
        })
    })
}

#[doc(hidden)]
pub unsafe extern "C" fn log_ttyout<P: IoPlugin, S: IoState<P>>(
    buf: *const raw::c_char,
    len:        raw::c_uint,
) -> raw::c_int {
    catch_unwind_log(|| {
        let env    = S::io_env();
        let plugin = S::io_plugin();

        if !env.command_info.iolog_ttyout && !P::IGNORE_IOLOG_HINTS {
            return Ok(());
        }

        let slice = ::std::slice::from_raw_parts(
            buf as *const _,
            len as _,
        );

        plugin.log_ttyout(slice).map_err(|err| {
            let _ = env.stderr().write_error(&err);
            err
        })
    })
}

#[doc(hidden)]
pub unsafe extern "C" fn log_stdin<P: IoPlugin, S: IoState<P>>(
    buf: *const raw::c_char,
    len:        raw::c_uint,
) -> raw::c_int {
    catch_unwind_log(|| {
        let env    = S::io_env();
        let plugin = S::io_plugin();

        if !env.command_info.iolog_stdin && !P::IGNORE_IOLOG_HINTS {
            return Ok(());
        }

        let slice = ::std::slice::from_raw_parts(
            buf as *const _,
            len as _,
        );

        plugin.log_stdin(slice).map_err(|err| {
            let _ = env.stderr().write_error(&err);
            err
        })
    })
}

#[doc(hidden)]
pub unsafe extern "C" fn log_stdout<P: IoPlugin, S: IoState<P>>(
    buf: *const raw::c_char,
    len:        raw::c_uint,
) -> raw::c_int {
    catch_unwind_log(|| {
        let env    = S::io_env();
        let plugin = S::io_plugin();

        if !env.command_info.iolog_stdout && !P::IGNORE_IOLOG_HINTS {
            return Ok(());
        }

        let slice = ::std::slice::from_raw_parts(
            buf as *const _,
            len as _,
        );

        plugin.log_stdout(slice).map_err(|err| {
            let _ = env.stderr().write_error(&err);
            err
        })
    })
}

#[doc(hidden)]
pub unsafe extern "C" fn log_stderr<P: IoPlugin, S: IoState<P>>(
    buf: *const raw::c_char,
    len:        raw::c_uint,
) -> raw::c_int {
    catch_unwind_log(|| {
        let env    = S::io_env();
        let plugin = S::io_plugin();

        if !env.command_info.iolog_stderr && !P::IGNORE_IOLOG_HINTS {
            return Ok(());
        }

        let slice = ::std::slice::from_raw_parts(
            buf as *const _,
            len as _,
        );

        plugin.log_stderr(slice).map_err(|err| {
            let _ = env.stderr().write_error(&err);
            err
        })
    })
}
