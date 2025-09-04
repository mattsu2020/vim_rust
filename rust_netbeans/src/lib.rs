use once_cell::sync::Lazy;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

const PROTOCOL_VERSION: &str = "2.5";

static NB_DEBUG_FILE: Lazy<Mutex<Option<File>>> = Lazy::new(|| Mutex::new(None));
static NB_DLEVEL: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(0));

/// Initialize logging for NetBeans debugging.
///
/// `log_path` is an optional file path to append logs to. `level` controls the
/// verbosity but is currently unused beyond being stored.
pub fn nbdebug_log_init<P: AsRef<Path>>(log_path: Option<P>, level: u32) {
    if let Some(path) = log_path.map(|p| p.as_ref().to_path_buf()) {
        if let Ok(file) = OpenOptions::new().create(true).append(true).open(path) {
            *NB_DEBUG_FILE.lock().unwrap() = Some(file);
            *NB_DLEVEL.lock().unwrap() = level;
        }
    }
}

/// Write a message to the NetBeans debug log if it was initialised.
pub fn nbdbg(msg: &str) {
    if let Some(ref mut file) = *NB_DEBUG_FILE.lock().unwrap() {
        let _ = writeln!(file, "{}", msg);
        let _ = file.flush();
    }
}

/// Sleep for `wait_secs` seconds.  This mimics the original `nbdebug_wait`
/// helper which allowed attaching a debugger before startup completed.
pub fn nbdebug_wait(wait_secs: u64) {
    std::thread::sleep(Duration::from_secs(wait_secs));
}

/// Simple asynchronous NetBeans client.
///
/// The client connects to the NetBeans host and performs the minimal
/// authentication handshake used by the original C implementation.  Only the
/// pieces required by the tests are implemented here.
pub struct NetbeansClient {
    stream: TcpStream,
}

impl NetbeansClient {
    /// Connect to the NetBeans server at `host:port` and authenticate using
    /// `password`.
    pub async fn connect(host: &str, port: u16, password: &str) -> io::Result<Self> {
        let addr = format!("{}:{}", host, port);
        let mut stream = TcpStream::connect(addr).await?;

        let auth = format!("AUTH {}\n", password);
        stream.write_all(auth.as_bytes()).await?;
        let version = format!("0:version=0 \"{}\"\n", PROTOCOL_VERSION);
        stream.write_all(version.as_bytes()).await?;

        Ok(Self { stream })
    }

    /// Send a raw message to the NetBeans server.
    pub async fn send(&mut self, msg: &str) -> io::Result<()> {
        self.stream.write_all(msg.as_bytes()).await
    }

    /// Close the connection to the server.
    pub async fn close(mut self) -> io::Result<()> {
        self.stream.shutdown().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio::io::AsyncReadExt;

    /// Verify that `NetbeansClient::connect` performs the authentication
    /// handshake by sending the `AUTH` and version messages.
    #[tokio::test]
    async fn connect_and_handshake() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 128];
            let n = socket.read(&mut buf).await.unwrap();
            String::from_utf8_lossy(&buf[..n]).to_string()
        });

        let client = NetbeansClient::connect("127.0.0.1", addr.port(), "secret")
            .await
            .unwrap();
        client.close().await.unwrap();

        let received = server.await.unwrap();
        assert!(received.contains("AUTH secret"));
        assert!(received.contains("version=0"));
    }
}

