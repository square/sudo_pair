//! A prelude module that includes most of what's necessary to start
//! using this crate.

pub use crate::sudo_io_plugin;
pub use crate::errors::{Result, Error, ErrorKind};
pub use crate::plugin::{IoEnv, IoPlugin};
pub use crate::options::OptionMap;
