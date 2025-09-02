use std::io;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::time::timeout;

#[cfg(feature = "os_mswin")]
use std::os::windows::io::{AsRawSocket, RawSocket as RawFd};
#[cfg(not(feature = "os_mswin"))]
use std::os::unix::io::{AsRawFd, RawFd};

/// Channel wraps a `TcpStream` and provides timeout aware read and write
/// operations.  The API mimics the original `channel.c` behaviour but uses
/// async/await and Rust closures instead of callbacks.
pub struct Channel {
    stream: TcpStream,
    timeout: Duration,
}

impl Channel {
    /// Connect to `addr` and construct a `Channel` with the specified timeout.
    pub async fn connect<A: ToSocketAddrs>(addr: A, timeout: Duration) -> io::Result<Self> {
        let stream = timeout_io(timeout, TcpStream::connect(addr)).await?;
        Ok(Self { stream, timeout })
    }

    /// Read data from the channel and invoke `cb` with the bytes read.  The
    /// callback is only executed when some data is available.
    pub async fn channel_read<F>(&mut self, mut cb: F) -> io::Result<usize>
    where
        F: FnMut(&[u8]),
    {
        let mut buf = vec![0u8; 1024];
        let n = timeout_io(self.timeout, self.stream.read(&mut buf)).await?;
        if n > 0 {
            cb(&buf[..n]);
        }
        Ok(n)
    }

    /// Write `data` to the channel.
    pub async fn channel_write(&mut self, data: &[u8]) -> io::Result<()> {
        timeout_io(self.timeout, self.stream.write_all(data)).await
    }

    /// Return the raw file descriptor or socket handle.
    pub fn raw_handle(&self) -> RawFd {
        self.stream.as_raw_fd()
    }
}

async fn timeout_io<F, T>(dur: Duration, fut: F) -> io::Result<T>
where
    F: std::future::Future<Output = io::Result<T>>,
{
    timeout(dur, fut)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn read_write_roundtrip() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            socket.write_all(b"ping").await.unwrap();
            let mut buf = [0u8; 4];
            socket.read_exact(&mut buf).await.unwrap();
            assert_eq!(&buf, b"pong");
        });

        let mut chan = Channel::connect(&addr, Duration::from_secs(5))
            .await
            .unwrap();
        let mut got = Vec::new();
        chan.channel_read(|b| got.extend_from_slice(b))
            .await
            .unwrap();
        assert_eq!(got, b"ping");
        chan.channel_write(b"pong").await.unwrap();
        server.await.unwrap();
    }

    #[tokio::test]
    async fn read_timeout() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let _server = tokio::spawn(async move {
            let (_socket, _) = listener.accept().await.unwrap();
            // Do not send anything so the client times out.
            tokio::time::sleep(Duration::from_millis(200)).await;
        });

        let mut chan = Channel::connect(&addr, Duration::from_millis(50))
            .await
            .unwrap();
        let res = chan.channel_read(|_| {}).await;
        assert!(res.is_err());
    }
}
