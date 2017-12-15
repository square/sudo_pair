use super::version::Version;

use libc::c_int;

// create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        FfiNulError(::std::ffi::NulError);
        Io(::std::io::Error);
    }

    errors {
        UnsupportedApiVersion(cur: Version) {
            description("sudo doesn't support the minimum plugin API version required by this plugin"),
            display("sudo called this plugin with an API version of {}, but a minimum of {} is required", cur, Version::minimum())
        }

        Uninitialized {
            description("the plugin failed to initialize"),
            display("the plugin failed to initialize"),
        }

        Unauthorized(reason: String) {
            description("command unauthorized"),
            display("command unauthorized"),
        }

        MissingOption(name: String, key: String) {
            description("expected an option that wasn't present"),
            display("expected the option {}[{}]", name, key),
        }

        MissingCallback(name: String) {
            description("a required sudo callback function wasn't provided")
            display("the sudo callback {} wasn't provided", name)
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
