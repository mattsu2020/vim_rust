use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;

/// Start a simple asynchronous TCP server that echoes received data.
pub async fn start_server(addr: &SocketAddr) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            if let Ok(n) = socket.read(&mut buf).await {
                if n > 0 {
                    let _ = socket.write_all(&buf[..n]).await;
                }
            }
        });
    }
}

/// Connect to the server.
pub async fn connect(addr: &SocketAddr) -> tokio::io::Result<TcpStream> {
    TcpStream::connect(addr).await
}

/// Send a debug command to the connected stream.
pub async fn send_debug_command(stream: &mut TcpStream, cmd: &str) -> tokio::io::Result<()> {
    stream.write_all(cmd.as_bytes()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic_flow() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(addr).await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let (mut sock, _) = listener.accept().await.unwrap();
                tokio::spawn(async move {
                    let mut buf = [0u8; 1024];
                    let n = sock.read(&mut buf).await.unwrap();
                    sock.write_all(&buf[..n]).await.unwrap();
                });
            }
        });
        let mut stream = connect(&addr).await.unwrap();
        send_debug_command(&mut stream, "ping").await.unwrap();
    }
}
