use crate::sys;

use std::sync::{Arc, Mutex};
use sudo_plugin_sys::{sudo_conv_t, sudo_conv_message, sudo_conv_reply};
use std::io;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::slice;

/// ConvMsgType is the type of conversation promp as specified by 
/// the sudo plugin
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum ConvMsgType {
    /// Don't echo user input
    ConvPromptEchoOff  = sys::SUDO_CONV_PROMPT_ECHO_OFF, /* do not echo user input */
    /// Echo user input
    ConvPromptEchoOn   = sys::SUDO_CONV_PROMPT_ECHO_ON, /* echo user input */
    /// The prompt is an Error Message
    ConvErrorMsg       = sys::SUDO_CONV_ERROR_MSG, /* error message */
    /// The prompt is an Info message
    ConvInfoMsg        = sys::SUDO_CONV_INFO_MSG, /* informational message */
    /// Mask user input
    ConvPrompMask      = sys::SUDO_CONV_PROMPT_MASK, /* mask user input */
    /// Allows for echo if no TTY
    ConvPromptEchoOk   = sys::SUDO_CONV_PROMPT_ECHO_OK, /* flag: allow echo if no tty */
    // This is only available on plugin version 1.14 which isn't supported yet
    //ConvPreferTTY      = sys::SUDO_CONV_PREFER_TTY, /* flag: use tty if possible */
}

/// ConversationPrompt is the struct that holds the actual prompt displayed
/// to the user. 
#[derive(Clone, Debug)]
pub struct ConversationPrompt {
    /// The type of prompt
    pub msg_type: ConvMsgType,
    /// The timeout for the prompt
    pub timeout: i32,
    /// The message to be displayed
    pub msg: String
}

/// ConversationReply is the reply (if any) from the user to our promt
#[derive(Clone, Debug)]
pub struct ConversationReply {
    /// The reply by the user
    pub reply: String
}
impl ConversationReply {
    fn from_conv_reply(scr: &sudo_conv_reply) -> Option<ConversationReply> {
        if scr.reply == ptr::null_mut() {
            return None;
        }
        unsafe { 
            return Some( ConversationReply {
                reply: CString::from_raw(scr.reply).into_string().expect("error converting reply to String")
            });
        }
    }
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

/// A facility implementing the Conversations API
#[derive(Clone, Debug)]
pub struct ConversationFacility {
    facility: Arc<Mutex<sudo_conv_t>>,
}

impl ConversationFacility {
    /// Constructs a new `ConversationFacility` that emits output and gets user input
    /// as part of the Conversations API exposed by sudo
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
        // TODO: do I need to forget this here?
        mem::forget(replies);

        // call the conversations API
        let count = unsafe {
            // (num_msgs, msgs[], replies[], callback*)
            (conv)(len, prompt_ptr, reply_ptr, ptr::null_mut())
        };

        let creplies: &[sudo_conv_reply] = unsafe {
            slice::from_raw_parts(reply_ptr, len as usize)
        };
        let replies = creplies.iter().map(|x| ConversationReply::from_conv_reply(x))
            .collect::<Vec<Option<ConversationReply>>>();

        

        // TODO: change to creating a real return value
        // unsafe {
        //     for reply in replies {
        //         print!("{:?}", CString::from_raw(reply.reply));
        //     }
        // }
        Ok(replies)
    }
}