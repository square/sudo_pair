//! A module that implements the conversations API for a sudo plugin.

use std::{convert::TryFrom, convert::TryInto, ffi::{CStr, NulError}, str::Utf8Error, sync::{Arc, Mutex}};
use libc::c_void;
use sudo_plugin_sys::{sudo_conv_t, sudo_conv_message, sudo_conv_reply};
use std::ffi::CString;
use thiserror::Error;
use super::MessageType;

#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ConversationError {
    #[error("too many prompts were provided")]
    TooManyPrompts,

    #[error("the conversation's reply couldn't be converted to a utf-8 string")]
    ConversationReplyString(#[source] Utf8Error),

    #[error("the conversation's prompt message couldn't be converted to a CString")]
    ConversationPromptMessage(#[source] NulError),

    #[error("the conversation's mutex couldn't be locked")]
    Lock,

    #[error("the conversation function pointer is NULL")]
    NullConversationFunction,

    #[error("the conversation reply pointer is NULL")]
    NullReply,

    #[error("the conversation API didn't return successfully")]
    CallFailure,
}

/// `ConversationPrompt` is the struct that holds the actual prompt displayed
/// to the user. 
#[derive(Clone, Debug)]
pub struct ConversationPrompt {
    /// The type of prompt.
    pub message_type: MessageType,
    /// The timeout for the prompt in seconds. 0 is no timeout.
    pub timeout: i32,
    /// The message to be displayed.
    pub message: String
}

impl TryFrom<ConversationPrompt> for sudo_conv_message {
    type Error = ConversationError;

    #[allow(clippy::cast_possible_wrap)]
    fn try_from(cp: ConversationPrompt) -> Result<Self, ConversationError> {
        Ok(sudo_conv_message {
            msg_type: cp.message_type.bits as i32,
            timeout: cp.timeout,
            // sudo does not take ownership of this.
            msg: CString::new(cp.message).map_err(ConversationError::ConversationPromptMessage)?.into_raw()
        })
    }
}

/// `ConversationReply` is the reply (if any) from the user to our promt
#[derive(Clone, Debug)]
pub struct ConversationReply {
    /// The reply by the user
    pub reply: String
}

impl TryFrom<sudo_conv_reply> for ConversationReply {
    type Error = ConversationError;

    fn try_from(scr: sudo_conv_reply) -> Result<ConversationReply, ConversationError> {
        if scr.reply.is_null() {
            return Ok(ConversationReply { reply: String::new() });
        }

        let result = unsafe { CStr::from_ptr(scr.reply) }.to_str();
        let result = match result {
            Ok(reply) => Ok(ConversationReply { reply: reply.into() }),
            Err(e) => {
                Err(ConversationError::ConversationReplyString(e))
            }
        };

        unsafe { libc::free(scr.reply as *mut c_void) };

        result
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
    ///
    /// # Safety
    ///
    /// This function *must* be provided with either a `None` or a real pointer
    /// to a `sudo_conv_t`-style function. Once provided to this function, the
    /// function pointer should be discarded and never used, as it is unsafe for
    /// this function to be called concurrently.
    #[must_use]
    pub(crate) fn new(conv: sudo_conv_t) -> Self {
        let conv = Arc::new(Mutex::new(conv));
        Self { facility: conv }
    }

    /// Take in a slice of `ConversationPrompts` and call the communicate() API
    /// exposed by the sudo plugin. Will return the corresponding replies.
    ///
    /// # Errors
    ///
    /// - The Mutex guarding the conversation function couldn't be locked.
    /// - Passing in strings which aren't convertable to `CString`s.
    /// - The conversation function call failed.
    /// - The user entered a string which isn't valid utf-8.
    pub fn communicate(&mut self, prompts: &[ConversationPrompt]) -> Result<Vec<ConversationReply>, ConversationError> {
        let len: i32 = prompts.len().try_into().map_err(|_| ConversationError::TooManyPrompts)?;

        let guard = self.facility.lock().or(Err(ConversationError::Lock))?;
        
        // check that a conversation pointer was provided
        let conv = guard.ok_or_else(|| ConversationError::NullConversationFunction)?;

        let mut cprompts = Vec::with_capacity(prompts.len());
        let mut replies = Vec::with_capacity(prompts.len());

        for prompt in prompts {
            cprompts.push(prompt.clone().try_into()?);
            replies.push(sudo_conv_reply { reply: std::ptr::null_mut() });
        }

        // call the conversations API and handle errors
        let cresult = unsafe {
            (conv)(len, cprompts.as_mut_ptr(), replies.as_mut_ptr(), std::ptr::null_mut())
        };

        if cresult == -1 {
            return Err(ConversationError::CallFailure);
        }

        replies.iter().map(|x| ConversationReply::try_from(*x))
            .collect::<Result<Vec<ConversationReply>, ConversationError>>()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_conversation_prompt() {
        let prompt = ConversationPrompt {
            message_type: MessageType::ECHO_ON,
            timeout: 200,
            message: "Hey".into(),
        };

        let message = sudo_conv_message::try_from(prompt).unwrap();
        #[allow(clippy::clippy::cast_possible_wrap)]
        assert_eq!(message.msg_type, MessageType::ECHO_ON.bits as i32);
        assert_eq!(message.timeout, 200);
        assert_eq!(unsafe { libc::strcmp(message.msg, b"Hey\0".as_ptr() as _) }, 0);

        unsafe { libc::free(message.msg as _) };
    }

    #[test]
    fn from_conversation_prompt_null() {
        let prompt = ConversationPrompt {
            message_type: MessageType::ERROR,
            timeout: 200,
            message: "\0".into(),
        };

        assert!(matches!(
            sudo_conv_message::try_from(prompt).unwrap_err(),
            ConversationError::ConversationPromptMessage(_)
        ));
    }

    #[test]
    fn from_conversation_reply() {
        let source = sudo_conv_reply {
            reply: unsafe { libc::strdup(b"Hey\0".as_ptr() as _) }
        };

        assert_eq!(
            ConversationReply::try_from(source).unwrap().reply,
            "Hey".to_string()
        );
    }

    #[test]
    fn from_conversation_reply_null() {
        let source = sudo_conv_reply {
            reply: std::ptr::null_mut(),
        };

        assert_eq!(
            ConversationReply::try_from(source).unwrap().reply,
            String::new()
        );
    }
}