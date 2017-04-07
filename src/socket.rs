use std::os::unix::ffi::OsStrExt;
use std::ffi::CString;
use std::fs;
use std::io::{Read, Write, Result, Error, ErrorKind};
use std::net::Shutdown;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;

use libc::{self, uid_t, gid_t, mode_t};

use unix_socket::{UnixListener, UnixStream, SocketAddr};

pub struct Socket {
    socket: UnixStream,
    _peer:   SocketAddr,
}

impl Socket {
    pub fn open<P: AsRef<Path>>(path: P, uid: uid_t, gid: gid_t, mode: mode_t) -> Result<Socket> {
        // if the path already exists as a socket, make a best-effort
        // attempt at unlinking it
        Self::unlink_socket(&path)?;

        // accept() on the socket will block until someone connects on
        // the other side of the socket
        let connection = UnixListener::bind(&path).and_then(|sock| {
            unsafe {
                let cpath = CString::new(path.as_ref().as_os_str().as_bytes())?;

                if libc::chown(
                    cpath.as_ptr(),
                    uid,
                    gid,
                ) == -1 {
                    return Err(Error::last_os_error());
                };

                if libc::chmod(
                    cpath.as_ptr(),
                    mode
                ) == -1 {
                    return Err(Error::last_os_error());
                }
            }

            sock.accept()
        })?;

        // once the connection has been made, we don't need the socket
        // to remain on the filesystem
        Self::unlink_socket(&path)?;

        return Ok(Socket{
            socket: connection.0,
            _peer:  connection.1,
        });
    }

    pub fn close(&mut self) -> Result<()> {
        self.socket.shutdown(Shutdown::Both)
    }

    // TODO: unlink when sudo_pair is ctrl-c'd
    fn unlink_socket<P: AsRef<Path>>(path: P) -> Result<()> {
        match fs::metadata(&path).map(|md| md.file_type().is_socket()) {
            // file exists, is a socket; delete it
            Ok(true) => fs::remove_file(&path),

            // file exists, is not a socket; abort
            Ok(false) => Err(Error::new(
                ErrorKind::AlreadyExists,
                format!(
                    "{} exists and is not a socket",
                    path.as_ref().to_string_lossy()
                )
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
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.socket.read(buf)
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.socket.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.socket.flush()
    }
}
