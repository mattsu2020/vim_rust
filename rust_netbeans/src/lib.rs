use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;
use std::sync::{Mutex, atomic::{AtomicU32, Ordering}};
use std::fs::{OpenOptions, File};
use std::io::Write;
use std::env;
use std::path::PathBuf;
use once_cell::sync::Lazy;
use std::time::Duration;
use std::thread;

static NB_DEBUG: Lazy<Mutex<Option<File>>> = Lazy::new(|| Mutex::new(None));
static NB_DLEVEL: AtomicU32 = AtomicU32::new(0);

pub const NB_TRACE: u32 = 0x0000_0001;
pub const WT_ENV: u32 = 0x1;
pub const WT_WAIT: u32 = 0x2;
pub const WT_STOP: u32 = 0x4;

/// Initialize logging if the environment variable given by `log_var`
/// points to a writable file. The debug level is taken from `level_var`.
pub fn nbdebug_log_init(log_var: &str, level_var: &str) {
    if let Ok(file) = env::var(log_var) {
        if let Ok(f) = OpenOptions::new().append(true).create(true).open(file) {
            if let Ok(mut guard) = NB_DEBUG.lock() {
                *guard = Some(f);
            }
        }
    }
    let level = env::var(level_var)
        .ok()
        .and_then(|s| u32::from_str_radix(&s, 0).ok())
        .unwrap_or(NB_TRACE);
    NB_DLEVEL.store(level, Ordering::Relaxed);
}

/// Write a debug message when tracing is enabled.
pub fn nbdbg(msg: &str) {
    if NB_DLEVEL.load(Ordering::Relaxed) & NB_TRACE != 0 {
        if let Ok(mut guard) = NB_DEBUG.lock() {
            if let Some(ref mut file) = *guard {
                let _ = writeln!(file, "{}", msg);
                let _ = file.flush();
            }
        }
    }
}

/// Optionally wait at startup for debugging.
pub fn nbdebug_wait(wait_flags: u32, wait_var: Option<&str>, wait_secs: u32) {
    if wait_flags & WT_ENV != 0 {
        if let Some(var) = wait_var {
            if let Some(secs) = env::var(var).ok().and_then(|v| v.parse::<u64>().ok()) {
                thread::sleep(Duration::from_secs(secs));
                return;
            }
        }
    } else if wait_flags & WT_WAIT != 0 {
        if lookup("~/.gvimwait") {
            let secs = if wait_secs > 0 && wait_secs < 120 { wait_secs } else { 20 };
            thread::sleep(Duration::from_secs(secs as u64));
            return;
        }
    } else if wait_flags & WT_STOP != 0 {
        if lookup("~/.gvimstop") {
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

fn lookup(file: &str) -> bool {
    let path = if file.starts_with("~") {
        if let Some(home) = env::home_dir() {
            let mut pb = PathBuf::from(home);
            if file.len() > 2 {
                pb.push(&file[2..]);
            }
            pb
        } else {
            PathBuf::from(file)
        }
    } else {
        PathBuf::from(file)
    };
    path.exists()
}

/// Simple asynchronous NetBeans-like client.
pub struct NetbeansClient {
    stream: TcpStream,
}

impl NetbeansClient {
    /// Connect to the given address.
    pub async fn connect(addr: &SocketAddr) -> tokio::io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }

    /// Send an event as bytes.
    pub async fn send_event(&mut self, data: &str) -> tokio::io::Result<()> {
        self.stream.write_all(data.as_bytes()).await
    }

    /// Receive an event into a string.
    pub async fn receive_event(&mut self) -> tokio::io::Result<Option<String>> {
        let mut buf = vec![0u8; 1024];
        let n = self.stream.read(&mut buf).await?;
        if n == 0 {
            return Ok(None);
        }
        Ok(Some(String::from_utf8_lossy(&buf[..n]).to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn basic_rpc_flow() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(addr).await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let n = sock.read(&mut buf).await.unwrap();
            sock.write_all(&buf[..n]).await.unwrap();
        });

        let mut client = NetbeansClient::connect(&addr).await.unwrap();
        client.send_event("ping").await.unwrap();
        let resp = client.receive_event().await.unwrap().unwrap();
        assert_eq!(resp, "ping");
    }
}
