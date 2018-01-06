use std::ffi::CString;
use std::fs;
use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;

use libc::{self, c_int, gid_t, mode_t, sighandler_t, uid_t};

use unix_socket::{SocketAddr, UnixListener, UnixStream};

pub(crate) struct Socket {
    socket: UnixStream,
    _peer:  SocketAddr,
}

impl Socket {
    pub(crate) fn open<P: AsRef<Path>>(
        path:   P,
        uid:    uid_t,
        gid:    gid_t,
        mode:   mode_t,
        ctrl_c: sighandler_t,
    ) -> io::Result<Self> {
        // if the path already exists as a socket, make a best-effort
        // attempt at unlinking it
        Self::unlink_socket(&path)?;

        // accept() on the socket will block until someone connects on
        // the other side of the socket
        let connection = UnixListener::bind(&path).and_then(|sock| {
            let cpath = CString::new(
                path.as_ref().as_os_str().as_bytes()
            )?;

            unsafe {
                if libc::chown(cpath.as_ptr(), uid, gid) == -1 {
                    return Err(io::Error::last_os_error());
                };

                if libc::chmod(cpath.as_ptr(), mode) == -1 {
                    return Err(io::Error::last_os_error());
                }

                // temporarily install the provided SIGINT handler while we
                // block on accept()
                let sigint = signal(libc::SIGINT, ctrl_c)?;
                let result = sock.accept();
                let _      = signal(libc::SIGINT, sigint)?;

                result
            }
        })?;

        // once the connection has been made, we don't need the socket
        // to remain on the filesystem
        Self::unlink_socket(&path)?;

        Ok(Self {
            socket: connection.0,
            _peer:  connection.1,
        })
    }

    pub(crate) fn close(&mut self) -> io::Result<()> {
        self.socket.shutdown(Shutdown::Both)
    }

    // TODO: unlink when sudo_pair is ctrl-c'd
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
        self.socket.read(buf)
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

unsafe fn signal(
    signum:  c_int,
    handler: sighandler_t,
) -> io::Result<sighandler_t> {
    match libc::signal(signum, handler) {
        libc::SIG_ERR => Err(io::Error::last_os_error()),
        previous      => Ok(previous),
    }
}
