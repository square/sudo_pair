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

//! Traits and structs directly used for implementation of plugins.

#![allow(clippy::module_name_repetitions)]

mod io_env;
mod io_plugin;
mod io_state;

pub use io_env::IoEnv;
pub use io_plugin::IoPlugin;
pub use io_state::IoState;
