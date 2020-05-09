#[derive(Clone, Debug)]
pub enum ConvMsgType {
    ConvPromptEchoOff  = 0x0001, /* do not echo user input */
    ConvPromptEchoOn   = 0x0002, /* echo user input */
    ConvErrorMsg       = 0x0003, /* error message */
    ConvInfoMsg        = 0x0004, /* informational message */
    ConvPrompMask      = 0x0005, /* mask user input */
    ConvPromptEchoOk   = 0x1000, /* flag: allow echo if no tty */
    ConvPreferTTY      = 0x2000, /* flag: use tty if possible */
}

pub struct ConversationPrompt {
    pub msg_type: ConvMsgType,
    pub timeout: i32,
    pub msg: String
}