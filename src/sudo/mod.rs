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

mod sudo_plugin;

pub use self::sudo_plugin::*;

use std::ffi::CString;
use std::io::{Result, Error, ErrorKind};

static mut SUDO_CONVERSATION: Option<sudo_conv_t>   = None;
static mut SUDO_PRINTF:       Option<sudo_printf_t> = None;

pub fn init(
    conversation: sudo_conv_t,
    printf:       sudo_printf_t,
) {
    unsafe {
        SUDO_CONVERSATION = Some(conversation);
        SUDO_PRINTF       = Some(printf);
    }
}

pub fn print(level: SUDO_CONV_FLAGS, message: &str) -> Result<()> {
    unsafe {
        SUDO_PRINTF.map(|printf| {
            let cstr = CString::new(message.as_bytes())?;
            let ret  = printf(level.bits(), cstr.as_ptr());

            match ret {
                -1 => Err(Error::new(ErrorKind::Other, "sudo_printf failed")),
                _  => Ok(()),
            }
        }).unwrap_or(Ok(()))
    }
}
