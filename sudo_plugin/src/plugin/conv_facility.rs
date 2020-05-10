use crate::sys;

use std::sync::{Arc, Mutex};
use sudo_plugin_sys::sudo_conv_t;

#[derive(Clone, Debug)]
#[repr(u32)]
pub enum ConvMsgType {
    ConvPromptEchoOff  = sys::SUDO_CONV_PROMPT_ECHO_OFF, /* do not echo user input */
    ConvPromptEchoOn   = sys::SUDO_CONV_PROMPT_ECHO_ON, /* echo user input */
    ConvErrorMsg       = sys::SUDO_CONV_ERROR_MSG, /* error message */
    ConvInfoMsg        = sys::SUDO_CONV_INFO_MSG, /* informational message */
    ConvPrompMask      = sys::SUDO_CONV_PROMPT_MASK, /* mask user input */
    ConvPromptEchoOk   = sys::SUDO_CONV_PROMPT_ECHO_OK, /* flag: allow echo if no tty */
    // This is only available on plugin version 1.14 which isn't supported yet
    //ConvPreferTTY      = sys::SUDO_CONV_PREFER_TTY, /* flag: use tty if possible */
}

pub struct ConversationPrompt {
    pub msg_type: ConvMsgType,
    pub timeout: i32,
    pub msg: String
}

pub struct ConversationFacility {
    facility: Arc<Mutex<sudo_conv_t>>,
}