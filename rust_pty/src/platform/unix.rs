use nix::pty::openpty;
pub use nix::pty::OpenptyResult;
use nix::unistd::{read, write};
use std::io;
use tokio::task;

pub fn open() -> io::Result<OpenptyResult> {
    openpty(None, None).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

pub async fn echo_loop(fd: i32) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    loop {
        let n = task::spawn_blocking({
            let mut b = buf;
            move || {
                let res = read(fd, &mut b);
                res.map(|n| (n, b))
            }
        })
        .await
        .unwrap()?;
        let (n, tmp) = n;
        buf = tmp;
        if n == 0 {
            break;
        }
        task::spawn_blocking({
            let data = buf[..n].to_vec();
            move || write(fd, &data).map(|_| ())
        })
        .await
        .unwrap()?;
    }
    Ok(())
}
