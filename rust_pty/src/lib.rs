use nix::errno::Errno;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::pty::{openpty, OpenptyResult};
use nix::unistd::{read, write};
use std::io;
use std::os::unix::io::{AsRawFd, OwnedFd};
use tokio::io::unix::AsyncFd;

/// Open a new pseudo terminal using `nix`.
pub fn open() -> io::Result<OpenptyResult> {
    openpty(None, None).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Echo data back on the file descriptor using non-blocking I/O with `AsyncFd`.
pub async fn echo_loop(fd: OwnedFd) -> io::Result<()> {
    fn nix_to_io(err: nix::Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err)
    }

    let flags =
        OFlag::from_bits_truncate(fcntl(fd.as_raw_fd(), FcntlArg::F_GETFL).map_err(nix_to_io)?);
    let mut new_flags = flags;
    new_flags.insert(OFlag::O_NONBLOCK);
    fcntl(fd.as_raw_fd(), FcntlArg::F_SETFL(new_flags)).map_err(nix_to_io)?;

    let async_fd = AsyncFd::new(fd)?;
    let mut buf = [0u8; 1024];
    loop {
        let mut read_guard = async_fd.readable().await?;
        match read(async_fd.as_raw_fd(), &mut buf) {
            Ok(0) => break,
            Ok(n) => {
                read_guard.clear_ready();
                let mut written = 0;
                while written < n {
                    let mut write_guard = async_fd.writable().await?;
                    match write(async_fd.as_raw_fd(), &buf[written..n]) {
                        Ok(m) => {
                            written += m;
                            write_guard.clear_ready();
                        }
                        Err(e) if e == Errno::EWOULDBLOCK => {
                            write_guard.clear_ready();
                            continue;
                        }
                        Err(e) => return Err(nix_to_io(e)),
                    }
                }
            }
            Err(e) if e == Errno::EWOULDBLOCK => {
                read_guard.clear_ready();
                continue;
            }
            Err(e) => return Err(nix_to_io(e)),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix::pty::OpenptyResult;
    use nix::unistd::{read, write};
    use std::os::unix::io::AsRawFd;
    use tokio::task;

    #[tokio::test]
    async fn echo_loop_echoes_input() {
        let OpenptyResult { master, slave } = open().unwrap();
        let handle = task::spawn(async move { echo_loop(slave).await.unwrap() });

        let msg = b"ping";
        write(master.as_raw_fd(), msg).unwrap();
        let mut buf = [0u8; 4];
        let n = read(master.as_raw_fd(), &mut buf).unwrap();
        assert_eq!(&buf[..n], msg);

        drop(master);
        handle.await.unwrap();
    }
}
