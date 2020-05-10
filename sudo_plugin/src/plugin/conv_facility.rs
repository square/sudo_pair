use crate::sys;

use std::sync::{Arc, Mutex};
use sudo_plugin_sys::{sudo_conv_t, sudo_conv_message};
use std::io;
use std::ffi::CString;

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

impl ConversationPrompt {
    fn convert_to_conv_message(self) -> Result<sudo_conv_message, &'static str> {
        let message = CString::new(self.msg).expect("Couldn't create cstring");

        Ok(sudo_conv_message {
            msg_type: self.msg_type as i32,
            timeout: self.timeout,
            msg: message.as_ptr()
        })
    } 
}

pub struct ConversationFacility {
    facility: Arc<Mutex<sudo_conv_t>>,
}

impl ConversationFacility {
    #[must_use]
    pub unsafe fn new(conv: sudo_conv_t) -> Self {
        let conv = Arc::new(Mutex::new(conv));
        Self { facility: conv }
    }

    pub fn communicate(&mut self, mut prompts: Vec<ConversationPrompt>) -> io::Result<()> {
        let guard = self.facility.lock().map_err(|_err|
            io::Error::new(io::ErrorKind::Other, "couldn't aquire conversation mutex")
        )?;
        
        // check that a conversation pointer was provided
        let _conv = guard.ok_or_else(||
            io::Error::new(io::ErrorKind::NotConnected, "no conv pntr provided")
        )?;

        // convert ConversationPrompt to sudo_conv_message and store it in an array
        // also create a cstring to store the responses
        prompts.shrink_to_fit();
        let ptr = prompts.as_mut_ptr();
        let len = prompts.len();
        
        // call the conversations API

        Ok(())
    }
}