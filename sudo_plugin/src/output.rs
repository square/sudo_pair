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

//! A module that includes input and output modules for sudo plugin .

pub(crate) mod print_facility;
pub(crate) mod tty;
pub mod conv_facility;

pub(crate) use print_facility::PrintFacility;
pub(crate) use tty::Tty;
pub(crate) use conv_facility::ConversationFacility;

use crate::sys;

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
enum Level {
    Info  = sys::SUDO_CONV_INFO_MSG,
    Error = sys::SUDO_CONV_PROMPT_ECHO_OFF,
}

bitflags::bitflags! {
    /// If used with the sudo conversation function and you want a reply from
    /// the user, you must specify one of the echo bits.
    pub struct MessageType: u32 {
        /// Do not echo user input.
        const ECHO_OFF       = sys::SUDO_CONV_PROMPT_ECHO_OFF;
        /// Echo user input.
        const ECHO_ON        = sys::SUDO_CONV_PROMPT_ECHO_ON;
        /// Error message.
        const ERROR          = sys::SUDO_CONV_ERROR_MSG;
        /// Informational message.
        const INFO           = sys::SUDO_CONV_INFO_MSG;
        /// Mask user input.
        const PROMPT_MASK    = sys::SUDO_CONV_PROMPT_MASK;
        /// Echo user input if no TTY.
        const PROMPT_ECHO_OK = sys::SUDO_CONV_PROMPT_ECHO_OK;
    }
}
