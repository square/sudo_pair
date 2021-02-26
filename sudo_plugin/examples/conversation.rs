use std::io::Write;
use sudo_plugin::{
    output::{conv_facility::ConversationPrompt, MessageType},
    prelude::*,
};

sudo_io_plugin! { conversation: Conversation }

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Some plugin error occurred")]
    PluginError(#[from] sudo_plugin::errors::Error),
}

impl Into<OpenStatus> for Error {
    fn into(self) -> OpenStatus {
        todo!()
    }
}

impl Into<LogStatus> for Error {
    fn into(self) -> LogStatus {
        todo!()
    }
}

struct Conversation;

impl IoPlugin for Conversation {
    type Error = Error;

    const NAME: &'static str = "conversation";

    fn open(env: &'static IoEnv) -> Result<Self, Self::Error> {
        let mut conv = env.conversation();

        let replies = conv
            .communicate(&[
                ConversationPrompt {
                    message_type: MessageType::ECHO_ON,
                    timeout: 10,
                    message: "Are you sure that's wise? ".into(),
                },
                ConversationPrompt {
                    message_type: MessageType::ECHO_ON,
                    timeout: 10,
                    message: "Do you want to reconsider? ".into(),
                },
            ])
            .unwrap();

        writeln!(env.stdout(), "Your replies were:").unwrap();
        for reply in replies {
            writeln!(env.stdout(), "- {}", reply.reply).unwrap();
        }

        Ok(Self {})
    }
}
