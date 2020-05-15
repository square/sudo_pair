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

use super::{IoEnv, IoPlugin};

/// This trait is implemented invisibly by the `sudo_io_plugin` macro
/// and is not user-visible.
#[doc(hidden)]
pub trait IoState<P: IoPlugin> {
    /// Returns a mutable borrow to the static location of the [`IoEnv`](IoEnv).
    ///
    /// # Safety
    ///
    /// This function is inherently unsafe since it returns a mutable
    /// reference to static memory. `sudo_plugin` never uses threads, so
    /// as long as implementors don't this will effectively be safe.
    unsafe fn io_env() -> &'static mut Option<IoEnv>;

    /// Returns a mutable borrow to the static location of the
    /// [`IoPlugin`](IoPlugin). We must take effort to guarantee that any
    /// time this method is called, the static `IoEnv` has been initialized
    /// (since the plugin may wish to operate on the `IoEnv` it is passed).
    ///
    /// # Safety
    ///
    /// This function is inherently unsafe since it returns a mutable
    /// reference to static memory. `sudo_plugin` never uses threads, so
    /// as long as implementors don't this will effectively be safe.
    unsafe fn io_plugin() -> &'static mut Option<P>;
}
