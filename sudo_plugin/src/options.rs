// Copyright 2020 Square Inc.
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

//! Parsers for the various key/value sets of options passed in by
//! `sudo_plugin`. These are parsed into a generic `OptionMap` which is
//! a thin convenience wrapper around a `HashMap<Vec<u8>, Vec<u8>>`
//! (keys and values are not guaranteed to be UTF-8 strings).
//!
//! Once parsed into the generic `OptionMap`, well-known sets of options
//! (`command_info`, `settings`, and `user_info`) are parsed into
//! structs with  values of the correct type type (e.g., `user_info.uid`
//! is a `uid_t`).


#[doc(hidden)] pub mod command_info;
#[doc(hidden)] pub mod option_map;
#[doc(hidden)] pub mod settings;
#[doc(hidden)] pub mod user_info;

mod traits;

pub use command_info::CommandInfo;
pub use option_map::OptionMap;
pub use settings::Settings;
pub use user_info::UserInfo;
