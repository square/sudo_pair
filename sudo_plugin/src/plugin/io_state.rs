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
use crate::core::OpenStatus;

/// This trait is implemented invisibly by the `sudo_io_plugin` macro
/// and is not user-visible.
///
/// # Safety
///
/// Implementing this trait is unsafe since it operates directly on
/// mutable static state. Efforts are currently underway to minimize the
/// total surface area of this unsafety.
#[doc(hidden)]
pub unsafe trait IoState<P: IoPlugin> {
    /// Returns a mutable borrow to a static location of the
    /// [`IoEnv`](IoEnv).
    ///
    /// # Safety
    ///
    /// This function is inherently unsafe since it returns a mutable
    /// reference to static memory. `sudo_plugin` never uses threads, so
    /// as long as implementors don't this will effectively be safe.
    unsafe fn io_env_mut() -> &'static mut Option<IoEnv>;

    /// Returns a mutable borrow to a static location of the type
    /// implementing [`IoPlugin`](IoPlugin).
    ///
    /// # Safety
    ///
    /// This function is inherently unsafe since it returns a mutable
    /// reference to static memory. `sudo_plugin` never uses threads, so
    /// as long as implementors don't this will effectively be safe.
    unsafe fn io_plugin_mut() -> &'static mut Option<P>;

    /// Initializes the static [`IoEnv`](IoEnv) containing the plugin
    /// environment. Takes a callback that is provided a reference to
    /// this environment and which must return a fully initialized
    /// [`IoPlugin`](IoPlugin).
    //
    // TODO: It feels leaky for this to care about OpenStatus.
    fn init<E: Into<OpenStatus>, F: FnOnce(&'static IoEnv) -> Result<P, E>> (
        env: IoEnv,
        f:   F,
    ) -> OpenStatus {
        unsafe {
            let env    = Self::io_env_mut().get_or_insert(env);
            let plugin = f(env).map(|p| Self::io_plugin_mut().get_or_insert(p));

            match plugin {
                Ok(_)  => OpenStatus::Ok,
                Err(e) => e.into(),
            }
        }
    }

    /// Drops the static [`IoPlugin`](IoPlugin) and [`IoEnv`](IoEnv) so
    /// they are properly cleaned up.
    ///
    /// # Safety
    ///
    /// This method is unsafe if anything is still holding on to the
    /// static references returned by other methods of this trait.
    /// Obviously this is Very Bad(TM), so this trait is being reworked
    /// to not provide access to mutable static data.
    unsafe fn drop<F: FnOnce(P)>(f: F) {
        // SAFETY: it's extremely important that the plugin be dropped
        // *before* the environment is dropped, because the plugin may
        // hold onto a reference to the static environment
        let _ = Self::io_plugin_mut().take().map(f);
        let _ = Self::io_env_mut()   .take();
    }

    /// Returns an immutable borrow to the static location of the
    /// contained [`IoEnv`](IoEnv). If the `IoState` has not been
    /// initialized yet, this method will panic.
    #[must_use]
    fn io_env() -> &'static IoEnv {
        unsafe {
            Self::io_env_mut().as_ref().expect("plugin environment was not initialized")
        }
    }

    /// Returns an immutable borrow to the static location of the
    /// contained [`IoPlugin`](IoPlugin). If the `IoState` has not been
    /// initialized yet, this method will panic.
    #[must_use]
    fn io_plugin() -> &'static P {
        unsafe {
            Self::io_plugin_mut().as_ref().expect("plugin was not initialized")
        }
    }
}
