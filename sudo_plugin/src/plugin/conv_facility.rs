use crate::sys;

use std::sync::{Arc, Mutex};
use sudo_plugin_sys::{sudo_conv_t, sudo_conv_message, sudo_conv_reply};
use std::io;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::slice;

/// ConvMsgType is the type of conversation prompt as specified by 
/// the sudo plugin
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum ConvMsgType {
    /// Do not echo user input
    ConvPromptEchoOff  = sys::SUDO_CONV_PROMPT_ECHO_OFF,
    /// Echo user input
    ConvPromptEchoOn   = sys::SUDO_CONV_PROMPT_ECHO_ON,
    /// The prompt is an Error Message
    ConvErrorMsg       = sys::SUDO_CONV_ERROR_MSG,
    /// The prompt is an informational message
    ConvInfoMsg        = sys::SUDO_CONV_INFO_MSG,
    /// Mask user input
    ConvPrompMask      = sys::SUDO_CONV_PROMPT_MASK,
    /// Allows for echo if no TTY
    ConvPromptEchoOk   = sys::SUDO_CONV_PROMPT_ECHO_OK,
    // This is only available on plugin version 1.14 which isn't supported yet
    //ConvPreferTTY      = sys::SUDO_CONV_PREFER_TTY, /* flag: use tty if possible */
}

/// ConversationPrompt is the struct that holds the actual prompt displayed
/// to the user. 
#[derive(Clone, Debug)]
pub struct ConversationPrompt {
    /// The type of prompt
    pub msg_type: ConvMsgType,
    /// The timeout for the prompt. 0 is no timeout
    pub timeout: i32,
    /// The message to be displayed
    pub msg: String
}

impl ConversationPrompt {
    /// This is an internal method for converting a ConversationPrompt to the
    /// sudo_conv_message type for FFI
    fn convert_to_conv_message(&self) -> io::Result<sudo_conv_message> {
        // TODO: can I get rid of this clone?
        let message = CString::new(self.msg.clone()).map_err(|err|
            io::Error::new(io::ErrorKind::Other, err)
        )?;

        Ok(sudo_conv_message {
            msg_type: self.msg_type as i32,
            timeout: self.timeout,
            // msg: message.as_ptr()
            msg: message.into_raw()
        })
    } 
}

/// ConversationReply is the reply (if any) from the user to our promt
#[derive(Clone, Debug)]
pub struct ConversationReply {
    /// The reply by the user
    pub reply: String
}

impl ConversationReply {
    /// Internal method for converting sudo_conv_reply to Option<ConversationReply> to expose only safe APIs
    fn from_conv_reply(scr: sudo_conv_reply) -> Option<ConversationReply> {
        if scr.reply.is_null() {
            return None;
        }
        unsafe { 
            Some( ConversationReply {
                reply: CString::from_raw(scr.reply).into_string().expect("error converting reply to String")
            })
        }
    }
}

/// A facility implementing the Conversations API
#[derive(Clone, Debug)]
pub struct ConversationFacility {
    facility: Arc<Mutex<sudo_conv_t>>,
}

impl ConversationFacility {
    /// Constructs a new `ConversationFacility` that emits output and gets user input
    /// as part of the Conversations API exposed by sudo
    /// # Safety
    ///
    /// This function *must* be provided with either a `None` or a real pointer
    /// to a `sudo_conv_t`-style function. Once provided to this function, the
    /// function pointer should be discarded at never used, as it is unsafe for
    /// this function to be called concurrently.
    #[must_use]
    pub unsafe fn new(conv: sudo_conv_t) -> Self {
        let conv = Arc::new(Mutex::new(conv));
        Self { facility: conv }
    }

    /// Take in a slice of ConversationPrompts and call the communicate() API exposed by
    /// the sudo plugin. Will return a slice of ConversationReply
    pub fn communicate(&mut self, prompts: &[ConversationPrompt]) -> io::Result<Vec<Option<ConversationReply>>> {
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
            replies.push(sudo_conv_reply {
                reply: ptr::null_mut()
            });
        }
        replies.shrink_to_fit();
        let reply_ptr = replies.as_mut_ptr();
        // Make sure replies doesn't get deallocated
        mem::forget(replies);

        // call the conversations API and handle errors
        let cresult = unsafe {
            (conv)(len, prompt_ptr, reply_ptr, ptr::null_mut())
        };
        if cresult == -1 {
            return Err(io::Error::new(io::ErrorKind::Other, "Error calling conversation API"));
        }
        // Convert the replies into ConversationReply structs and return
        let creplies: &[sudo_conv_reply] = unsafe {
            slice::from_raw_parts(reply_ptr, len as usize)
        };
        let replies = creplies.iter().map(|x| ConversationReply::from_conv_reply(*x))
            .collect::<Vec<Option<ConversationReply>>>();
        Ok(replies)
    }
}