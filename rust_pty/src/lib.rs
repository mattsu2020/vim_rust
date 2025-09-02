use nix::pty::{openpty, OpenptyResult};
use std::io;
use tokio::task;
use nix::unistd::{read, write};

/// Open a new pseudo terminal using `nix`.
pub fn open() -> io::Result<OpenptyResult> {
    openpty(None, None).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Echo data back on the file descriptor using blocking reads wrapped in `tokio`.
pub async fn echo_loop(fd: i32) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    loop {
        let n = task::spawn_blocking(move || read(fd, &mut buf)).await.unwrap()?;
        if n == 0 { break; }
        task::spawn_blocking(move || write(fd, &buf[..n])).await.unwrap()?;
    }
    Ok(())
}
