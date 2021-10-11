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

//! A prelude module that includes most of what's necessary to start
//! using this crate.

pub use crate::core::{OpenStatus, LogStatus};
pub use crate::errors::Error;
pub use crate::options::OptionMap;
pub use crate::plugin::{IoEnv, IoPlugin};
pub use crate::sudo_io_plugin;
