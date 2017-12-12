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

use libc::c_int;

// create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        FfiNulError(::std::ffi::NulError);
        Io(::std::io::Error);
    }

    errors {
        Uninitialized {
            description("the plugin failed to initialize"),
            display("the plugin failed to initialize"),
        }

        Unauthorized(reason: String) {
            description("command unauthorized"),
            display("command unauthorized: {}", reason),
        }

        MissingOption(name: String, key: String) {
            description("expected an option that wasn't present"),
            display("expected the option {}[{}]", name, key),
        }
    }
}

pub trait AsSudoPluginRetval {
    fn as_sudo_plugin_retval(&self) -> c_int;
}

impl<T> AsSudoPluginRetval for Result<T> {
    fn as_sudo_plugin_retval(&self) -> c_int {
        match *self {
            Ok(_)      =>  1,
            Err(ref e) => e.as_sudo_plugin_retval(),
        }
    }
}

impl AsSudoPluginRetval for Error {
    fn as_sudo_plugin_retval(&self) -> c_int {
        match *self {
            Error(ErrorKind::Unauthorized(_), _) => 0,
            Error(_, _)                          => -1,
        }
    }
}
