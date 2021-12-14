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

// these warnings are unavoidable with names like `uid` and `gid`, and
// such names are natural to use for this problem domain so should not
// be avoided
#![allow(clippy::similar_names)]

use std::ffi::CString;
use std::fs;
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::mem;
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::prelude::*;
use std::path::Path;
use std::ptr;

use libc::{self, gid_t, mode_t, uid_t};

#[derive(Debug)]
pub(crate) struct Socket {
    socket: UnixStream,
}

impl Socket {
    pub(crate) fn open<P: AsRef<Path>>(
        path: P,
        uid: uid_t,
        gid: gid_t,
        mode: mode_t,
    ) -> Result<Self> {
        let path = path.as_ref();

        Self::enforce_ownership(path)?;

        // if the path already exists as a socket, make a best-effort
        // attempt at unlinking it
        Self::unlink(path)?;

        // by default, ensure no permissions on the created socket since
        // we're going to customize them immediately afterward
        let umask = unsafe { libc::umask(libc::S_IRWXU | libc::S_IRWXG | libc::S_IRWXO) };

        let socket = UnixListener::bind(&path).and_then(|listener| {
            let cpath = CString::new(path.as_os_str().as_bytes())?;

            unsafe {
                if libc::chown(cpath.as_ptr(), uid, gid) == -1 {
                    return Err(Error::last_os_error());
                };

                if libc::chmod(cpath.as_ptr(), mode) == -1 {
                    return Err(Error::last_os_error());
                }

                let fd = listener.as_raw_fd();
                let mut readfds = mem::MaybeUninit::<libc::fd_set>::uninit();

                libc::FD_ZERO(readfds.as_mut_ptr());

                let mut readfds = readfds.assume_init();

                libc::FD_SET(fd, &mut readfds);

                // rust automatically wraps the `accept()` function in a
                // loop that retries on SIGINT, so we have to get
                // creative here and `select(2)` ourselves if we want
                // Ctrl-C to interrupt the process
                match libc::select(
                    fd + 1, // this must be greater than the fd's int value
                    &mut readfds,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                ) {
                    1 => (),
                    -1 => return Err(Error::last_os_error()),
                    0 => unreachable!("`select` returned 0 even though no timeout was set"),
                    _ => unreachable!("`select` indicated that more than 1 fd is ready"),
                };

                // as a sanity check, confirm that the fd we're going to
                // `accept` is the one that `select` says is ready
                if !libc::FD_ISSET(fd, &readfds) {
                    unreachable!("`select` returned an unexpected file descriptor");
                }
            }

            listener.accept().map(|connection| Self { socket: connection.0 })
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
        let _ = Self::unlink(path);

        // restore the process' original umask
        let _ = unsafe { libc::umask(umask) };

        socket
    }

    pub(crate) fn close(&mut self) -> Result<()> {
        self.socket.shutdown(Shutdown::Both)
    }

    fn unlink(path: &Path) -> Result<()> {
        match fs::metadata(&path).map(|md| md.file_type().is_socket()) {
            // file exists, is a socket; delete it
            Ok(true) => fs::remove_file(path),

            // file exists, is not a socket; abort
            Ok(false) => Err(Error::new(
                ErrorKind::AlreadyExists,
                format!("{} exists and is not a socket", path.to_string_lossy()),
            )),

            // file doesn't exist; nothing to do
            _ => Ok(()),
        }
    }

    fn enforce_ownership(path: &Path) -> Result<()> {
        let parent = path.parent().ok_or_else(|| {
            Error::new(
                ErrorKind::AlreadyExists,
                format!(
                    "couldn't determine permissions of the parent directory for {}",
                    path.to_string_lossy()
                ),
            )
        })?;

        let parent = CString::new(parent.as_os_str().as_bytes())?;

        unsafe {
            let mut stat = mem::MaybeUninit::<libc::stat>::uninit();

            if libc::stat(parent.as_ptr(), stat.as_mut_ptr()) == -1 {
                return Err(Error::last_os_error());
            }

            let stat = stat.assume_init();

            if stat.st_mode & libc::S_IFDIR == 0 {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("the socket path {} is not a directory", parent.to_string_lossy(),),
                ));
            }

            if stat.st_uid != libc::geteuid() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!(
                        "the socket directory {} is not owned by root",
                        parent.to_string_lossy(),
                    ),
                ));
            }

            // TODO: temporarily disabled while I relearn everything I
            // know about POSIX filesystem ownership.
            //
            // All of this is to the best of my (current) understanding.
            // On Linux, new files are created with their uid and gid
            // set to the euid and egid of the process creating them. On
            // Darwin (and probably other BSDs), they are created with
            // the euid of the process creating them, but their egid is
            // that of the directory they're being created in (which is
            // typically behavior on Linux only if the setgid is enabled
            // on the directory).
            //
            // So I'd like to ensure the file is owned by root's primary
            // group, so that the created sockets don't inherit a group
            // that unprivileged users are in. But I'd first have to
            // actually figure out the primary group for my `euid`
            // (sudo is run *setuid*, which doesn't change the `egid`),
            // and then I'd have to... I don't know, check that nobody
            // else is in it? That doesn't seem like a lot of ROI on my
            // effort. So for now I'll just check that the group doesn't
            // have any permissions to this directory.
            //
            // if stat.st_gid != libc::getegid() {
            //     return Err(Error::new(ErrorKind::Other, format!(
            //         "the socket directory {} is not owned by root's group",
            //         parent.to_string_lossy(),
            //     )));
            // }

            if stat.st_mode & (libc::S_IWGRP | libc::S_IWOTH) != 0 {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!(
                        "the socket directory {} has insecure permissions",
                        parent.to_string_lossy(),
                    ),
                ));
            }
        }

        Ok(())
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // read() will block until someone writes on the other side
        // of the socket, so we ensure that the signal handler for
        // Ctrl-C aborts the read instead of restarting it
        // automatically
        ctrl_c_aborts_syscalls(|| self.socket.read(buf))?
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        ctrl_c_aborts_syscalls(|| self.socket.write(buf))?
    }

    fn flush(&mut self) -> Result<()> {
        ctrl_c_aborts_syscalls(|| self.socket.flush())?
    }
}

/// Sets up a handler for Ctrl-C (SIGINT) that's a no-op, but with the
/// `SA_RESTART` flag disabled, for the duration of the passed function
/// call.
///
/// Disabling `SA_RESTART` ensures that blocking calls like `accept(2)`
/// will be terminated upon receipt on the signal instead of
/// automatically resuming.
fn ctrl_c_aborts_syscalls<F, T>(func: F) -> Result<T>
where
    F: FnOnce() -> T,
{
    unsafe {
        let mut sigaction_old = mem::MaybeUninit::<libc::sigaction>::uninit();
        let sigaction_null = ::std::ptr::null_mut();

        // retrieve the existing handler
        sigaction(libc::SIGINT, sigaction_null, sigaction_old.as_mut_ptr())?;

        let sigaction_old = sigaction_old.assume_init();

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
}

/// Installs the new handler for the signal identified by `sig` if `new`
/// is non-null. Returns the preexisting handler for the signal if `old`
/// is non-null.
unsafe fn sigaction(
    sig: libc::c_int,
    new: *const libc::sigaction,
    old: *mut libc::sigaction,
) -> Result<()> {
    if libc::sigaction(sig, new, old) == -1 {
        return Err(Error::last_os_error());
    }

    Ok(())
}
