use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use std::os::raw::c_ushort;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Message {
    pub text: String,
}

pub async fn echo_server(port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];
            if let Ok(n) = socket.read(&mut buf).await {
                if n > 0 {
                    let _ = socket.write_all(&buf[..n]).await;
                }
            }
        });
    }
}

pub async fn send_message(port: u16, msg: &Message) -> std::io::Result<Message> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await?;
    let data = serde_json::to_vec(msg).unwrap();
    stream.write_all(&data).await?;
    let mut buf = vec![0u8; data.len()];
    stream.read_exact(&mut buf).await?;
    let reply: Message = serde_json::from_slice(&buf).unwrap();
    Ok(reply)
}

#[no_mangle]
pub extern "C" fn rs_start_echo_server(port: c_ushort) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _ = rt.block_on(async {
            // give up after server error
            let _ = echo_server(port).await;
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn roundtrip() {
        let port = 45454;
        tokio::spawn(async move {
            echo_server(port).await.unwrap();
        });
        sleep(Duration::from_millis(100)).await;
        let msg = Message { text: "hello".into() };
        let reply = send_message(port, &msg).await.unwrap();
        assert_eq!(reply, msg);
    }

    #[test]
    fn serde_compat() {
        let msg = Message { text: "test".into() };
        let json = serde_json::to_string(&msg).unwrap();
        let de: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, de);
    }
}
