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

use std::ffi::CString;
use std::fs;
use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;

use libc::{self, gid_t, mode_t, uid_t};

use unix_socket::{UnixListener, UnixStream};

#[derive(Debug)]
pub(crate) struct Socket {
    socket: UnixStream,
}

impl Socket {
    pub(crate) fn open<P: AsRef<Path>>(
        path:   P,
        uid:    uid_t,
        gid:    gid_t,
        mode:   mode_t,
    ) -> io::Result<Self> {
        let cpath = CString::new(
            path.as_ref().as_os_str().as_bytes()
        )?;

        // if the path already exists as a socket, make a best-effort
        // attempt at unlinking it
        Self::unlink_socket(&path)?;

        // create a socket and wait for someone to connect; we *don't*
        // suffix this with an immediate `?` so that we have a chance
        // to clean up after ourselves first
        let connection = UnixListener::bind(&path).and_then(|sock| {
            unsafe {
                if libc::chown(cpath.as_ptr(), uid, gid) == -1 {
                    return Err(io::Error::last_os_error());
                };

                if libc::chmod(cpath.as_ptr(), mode) == -1 {
                    return Err(io::Error::last_os_error());
                }

                // accept() will block until someone connects on the
                // other side of the socket, so we ensure that the
                // signal handler for Ctrl-C aborts the call instead of
                // restarting them automatically
                ctrl_c_aborts_syscalls(|| {
                    sock.accept()
                })?
            }
        });

        // once the connection has been made (or aborted due to ctrl-c),
        // we don't need the socket to remain on the filesystem
        //
        // we ignore the result of this operation (instead of returning
        // the error because a) any error from the previous operation is
        // of higher importance (we didn't return the error immediately
        // because we want to unlink the socket regardless) and b) it's
        // more important to continue the sudo session than to worry
        // about filesystem janitorial work
        let _ = Self::unlink_socket(&path);

        // unwrap the connection and return any errors now that we've
        // cleaned up after ourself
        let connection = connection?;

        Ok(Self {
            socket: connection.0,
        })
    }

    pub(crate) fn close(&mut self) -> io::Result<()> {
        self.socket.shutdown(Shutdown::Both)
    }

    fn unlink_socket<P: AsRef<Path>>(path: P) -> io::Result<()> {
        match fs::metadata(&path).map(|md| md.file_type().is_socket()) {
            // file exists, is a socket; delete it
            Ok(true) => fs::remove_file(&path),

            // file exists, is not a socket; abort
            Ok(false) => Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "{} exists and is not a socket",
                    path.as_ref().to_string_lossy()
                ),
            )),

            // file doesn't exist; nothing to do
            _ => Ok(()),
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            // read() will block until someone writes on the other side
            // of the socket, so we ensure that the signal handler for
            // Ctrl-C aborts the read instead of restarting it
            // automatically
            ctrl_c_aborts_syscalls(|| {
                self.socket.read(buf)
            })?
        }
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.socket.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.socket.flush()
    }
}

/// Sets up a handler for Ctrl-C (SIGINT) that's a no-op, but with the
/// `SA_RESTART` flag disabled, for the duration of the passed function
/// call.
///
/// Disabling `SA_RESTART` ensures that blocking calls like `accept(2)`
/// will be terminated upon receipt on the signal instead of
/// automatically resuming.
unsafe fn ctrl_c_aborts_syscalls<F, T>(func: F) -> io::Result<T>
    where F: FnOnce() -> T
{
    let mut sigaction_old  = ::std::mem::uninitialized();
    let     sigaction_null = ::std::ptr::null_mut();

    // retrieve the existing handler
    sigaction(libc::SIGINT, sigaction_null, &mut sigaction_old)?;

    // copy the old handler, but mask out SA_RESTART
    let mut sigaction_new = sigaction_old;
    sigaction_new.sa_flags &= !libc::SA_RESTART;

    // install the new handler
    sigaction(libc::SIGINT, &sigaction_new, sigaction_null)?;

    let result = func();

    // reinstall the old handler
    sigaction(libc::SIGINT, &sigaction_old, sigaction_null)?;

    Ok(result)
}

/// Installs the new handler for the signal identified by `sig` if `new`
/// is non-null. Returns the preexisting handler for the signal if `old`
/// is non-null.
unsafe fn sigaction(
    sig: libc::c_int,
    new: *const libc::sigaction,
    old: *mut   libc::sigaction,
) -> io::Result<()> {
    if libc::sigaction(sig, new, old) == -1 {
        return Err(io::Error::last_os_error())
    }

    Ok(())
}
