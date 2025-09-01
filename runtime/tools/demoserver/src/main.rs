use tokio::net::{TcpListener, tcp::OwnedWriteHalf};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, AsyncBufReadExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use serde_json::{Value, json};
use hyper::Version;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = Version::HTTP_11;

    let listener = TcpListener::bind("127.0.0.1:8765").await?;
    println!("Server loop running");
    println!("Listening on port 8765");

    let shared = Arc::new(Mutex::new(None::<OwnedWriteHalf>));

    {
        let shared = shared.clone();
        tokio::spawn(async move {
            let mut stdin = BufReader::new(tokio::io::stdin()).lines();
            while let Ok(Some(line)) = stdin.next_line().await {
                if line.contains("quit") {
                    println!("Goodbye!");
                    std::process::exit(0);
                }
                let mut guard = shared.lock().await;
                if let Some(stream) = guard.as_mut() {
                    if let Err(e) = stream.write_all(format!("{}\n", line).as_bytes()).await {
                        eprintln!("send failed: {e}");
                    }
                    if let Err(e) = stream.flush().await {
                        eprintln!("flush failed: {e}");
                    }
                } else {
                    println!("No socket yet");
                }
            }
        });
    }

    loop {
        let (socket, _) = listener.accept().await?;
        println!("=== socket opened ===");
        let (mut reader, writer) = socket.into_split();
        {
            let mut guard = shared.lock().await;
            *guard = Some(writer);
        }
        let shared_inner = shared.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => {
                        println!("=== socket closed ===");
                        break;
                    }
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]);
                        println!("received: {}", data);
                        let decoded: Result<Vec<Value>, _> = serde_json::from_str(&data);
                        let decoded = match decoded {
                            Ok(d) => d,
                            Err(_) => {
                                println!("json decoding failed");
                                continue;
                            }
                        };
                        let seq = decoded.get(0).and_then(|v| v.as_i64()).unwrap_or(-1);
                        if seq >= 0 {
                            let msg = decoded.get(1).and_then(|v| v.as_str()).unwrap_or("");
                            let (id, resp) = match msg {
                                "hello!" => (seq, "got it"),
                                "hello channel!" => (0, "got that"),
                                _ => (seq, "what?"),
                            };
                            let encoded = serde_json::to_string(&json!([id, resp])).unwrap();
                            println!("sending {}", encoded);
                            let mut guard = shared_inner.lock().await;
                            if let Some(stream) = guard.as_mut() {
                                if let Err(e) = stream.write_all(format!("{}\n", encoded).as_bytes()).await {
                                    eprintln!("send error: {e}");
                                    break;
                                }
                                if let Err(e) = stream.flush().await {
                                    eprintln!("flush error: {e}");
                                    break;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        println!("=== socket error ===");
                        break;
                    }
                }
            }
            let mut guard = shared_inner.lock().await;
            *guard = None;
        });
    }
}
