use crate::sys;

use std::sync::{Arc, Mutex};
use sudo_plugin_sys::{sudo_conv_t, sudo_conv_message, sudo_conv_reply};
use std::io;
use std::ffi::CString;
use std::mem;
use std::ptr;

#[derive(Copy, Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct ConversationPrompt {
    pub msg_type: ConvMsgType,
    pub timeout: i32,
    pub msg: String
}

#[derive(Clone, Debug)]
pub struct ConversationReply {
    pub reply: String
}

impl ConversationPrompt {
    fn convert_to_conv_message(&self) -> io::Result<sudo_conv_message> {
        // TODO: can I get rid of this clone?
        let message = CString::new(self.msg.clone()).map_err(|err|
            io::Error::new(io::ErrorKind::Other, err)
        )?;

        Ok(sudo_conv_message {
            msg_type: self.msg_type as i32,
            timeout: self.timeout,
            msg: message.as_ptr()
        })
    } 
}

#[derive(Clone, Debug)]
pub struct ConversationFacility {
    facility: Arc<Mutex<sudo_conv_t>>,
}

impl ConversationFacility {
    #[must_use]
    pub unsafe fn new(conv: sudo_conv_t) -> Self {
        let conv = Arc::new(Mutex::new(conv));
        Self { facility: conv }
    }

    pub fn communicate(&mut self, prompts: &[ConversationPrompt]) -> io::Result<()> {
        let guard = self.facility.lock().map_err(|_err|
            io::Error::new(io::ErrorKind::Other, "couldn't aquire conversation mutex")
        )?;
        
        // check that a conversation pointer was provided
        let conv = guard.ok_or_else(||
            io::Error::new(io::ErrorKind::NotConnected, "no conv pntr provided")
        )?;

        // convert ConversationPrompt to sudo_conv_message and store it in an array
        let mut sudo_conv_prompts: Vec<sudo_conv_message> = prompts.iter()
            .map(|x| x.convert_to_conv_message())
            .collect::<io::Result<Vec<sudo_conv_message>>>()?;
        sudo_conv_prompts.shrink_to_fit();
        let prompt_ptr = sudo_conv_prompts.as_mut_ptr();
        let len = sudo_conv_prompts.len() as i32;
        
        // make sure that sudo_conv_prompts doesn't get dealloced by rust
        mem::forget(sudo_conv_prompts);
        
        // make the responses vector
        let mut replies = Vec::new();
        for _ in 0..len {
            let reply_string = CString::new("").map_err(|err|
                io::Error::new(io::ErrorKind::InvalidData, err)
            )?;
            replies.push(sudo_conv_reply {
                reply: reply_string.into_raw()
            });
        }
        replies.shrink_to_fit();
        let reply_ptr = replies.as_mut_ptr();
        // TODO: do I need to forget this here?
        //mem::forget(replies);

        // call the conversations API
        let _count = unsafe {
            // (num_msgs, msgs[], replies[], callback*)
            (conv)(len, prompt_ptr, reply_ptr, ptr::null_mut())
        };

        // TODO: change to creating a real return value
        unsafe {
            for reply in replies {
                print!("{:?}", CString::from_raw(reply.reply));
            }
        }
        Ok(())
    }
}