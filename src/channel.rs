use std::io;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;
use std::process::Stdio;

#[cfg(not(target_os = "windows"))]
use std::os::unix::io::{AsRawFd, RawFd};
#[cfg(target_os = "windows")]
use std::os::windows::io::{AsRawSocket, RawSocket as RawFd};

enum ChannelKind {
    Tcp(TcpStream),
    Job {
        child: Child,
        stdin: Option<ChildStdin>,
        stdout: Option<ChildStdout>,
    },
}

/// Channel wraps either a `TcpStream` or the pipes of a spawned job and provides
/// timeout aware read/write operations with callback support.
pub struct Channel {
    kind: ChannelKind,
    timeout: Duration,
}

impl Channel {
    /// Connect to `addr` over TCP.
    pub async fn connect<A: ToSocketAddrs>(addr: A, timeout: Duration) -> io::Result<Self> {
        let stream = timeout_io(timeout, TcpStream::connect(addr)).await?;
        Ok(Self { kind: ChannelKind::Tcp(stream), timeout })
    }

    /// Spawn a job using `cmd`. Stdio pipes are used for communication.
    pub async fn spawn(cmd: &str, timeout: Duration) -> io::Result<Self> {
        let mut child = Command::new(cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        Ok(Self {
            kind: ChannelKind::Job { child, stdin, stdout },
            timeout,
        })
    }

    /// Read data from the channel and invoke `cb` with the bytes read.
    pub async fn channel_read<F>(&mut self, mut cb: F) -> io::Result<usize>
    where
        F: FnMut(&[u8]),
    {
        let mut buf = vec![0u8; 1024];
        let n = match &mut self.kind {
            ChannelKind::Tcp(stream) => timeout_io(self.timeout, stream.read(&mut buf)).await?,
            ChannelKind::Job { stdout, .. } => {
                if let Some(out) = stdout.as_mut() {
                    timeout_io(self.timeout, out.read(&mut buf)).await?
                } else {
                    0
                }
            }
        };
        if n > 0 {
            cb(&buf[..n]);
        }
        Ok(n)
    }

    /// Write `data` to the channel.
    pub async fn channel_write(&mut self, data: &[u8]) -> io::Result<()> {
        match &mut self.kind {
            ChannelKind::Tcp(stream) => {
                timeout_io(self.timeout, stream.write_all(data)).await
            }
            ChannelKind::Job { stdin, .. } => {
                if let Some(inp) = stdin.as_mut() {
                    timeout_io(self.timeout, inp.write_all(data)).await
                } else {
                    Err(io::Error::new(io::ErrorKind::BrokenPipe, "stdin closed"))
                }
            }
        }
    }

    /// Close the job's stdin handle, signaling EOF to the process.
    pub fn close_stdin(&mut self) {
        if let ChannelKind::Job { stdin, .. } = &mut self.kind {
            stdin.take();
        }
    }

    /// Wait for the job to finish and return the exit status.
    pub async fn wait(&mut self) -> io::Result<std::process::ExitStatus> {
        match &mut self.kind {
            ChannelKind::Job { child, .. } => timeout_io(self.timeout, child.wait()).await,
            _ => Err(io::Error::new(io::ErrorKind::Other, "not a job")),
        }
    }

    /// Return the raw file descriptor or socket handle if this is a TCP channel.
    pub fn raw_handle(&self) -> Option<RawFd> {
        match &self.kind {
            ChannelKind::Tcp(stream) => Some(stream.as_raw_fd()),
            _ => None,
        }
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

    #[tokio::test]
    async fn job_roundtrip() {
        let mut chan = Channel::spawn("cat", Duration::from_secs(5))
            .await
            .unwrap();
        chan.channel_write(b"hello\n").await.unwrap();
        chan.close_stdin();
        let mut got = Vec::new();
        chan.channel_read(|b| got.extend_from_slice(b)).await.unwrap();
        assert_eq!(got, b"hello\n");
        chan.wait().await.unwrap();
    }
}

