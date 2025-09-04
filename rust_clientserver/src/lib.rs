use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long};
use std::sync::Mutex;
use once_cell::sync::Lazy;

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

// ----------------- C ABI stubs for Vim's socket server API -----------------

static SERVER_STATE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

const OK: c_int = 1;
const FAIL: c_int = 0;

#[no_mangle]
pub extern "C" fn socket_server_init(_servername: *const u8) -> c_int {
    let mut st = SERVER_STATE.lock().unwrap();
    *st = true;
    OK
}

#[no_mangle]
pub extern "C" fn socket_server_uninit() {
    let mut st = SERVER_STATE.lock().unwrap();
    *st = false;
}

#[no_mangle]
pub extern "C" fn socket_server_valid() -> c_int {
    if *SERVER_STATE.lock().unwrap() { OK } else { FAIL }
}

#[no_mangle]
pub extern "C" fn socket_server_waiting_accept() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn socket_server_list_sockets() -> *mut u8 {
    // Return empty string
    let s = CString::new("").unwrap();
    s.into_raw() as *mut u8
}

#[no_mangle]
pub extern "C" fn socket_server_send(
    _servername: *const u8,
    _keys: *const u8,
    result: *mut *mut u8,
    client: *mut *mut u8,
    _expr_flag_or_ptr: c_long,
    _timeout_ms: c_int,
    _flags: c_int,
) -> c_int {
    unsafe {
        if !result.is_null() { *result = std::ptr::null_mut(); }
        if !client.is_null() { *client = std::ptr::null_mut(); }
    }
    FAIL
}

#[no_mangle]
pub extern "C" fn socket_server_read_reply(
    _receiver: *const u8,
    out: *mut *mut u8,
    _timeout_ms: c_int,
) -> c_int {
    unsafe { if !out.is_null() { *out = std::ptr::null_mut(); } }
    FAIL
}

#[no_mangle]
pub extern "C" fn socket_server_peek_reply(
    _serverid: *const u8,
    out: *mut *mut u8,
) -> c_int {
    unsafe { if !out.is_null() { *out = std::ptr::null_mut(); } }
    FAIL
}

#[no_mangle]
pub extern "C" fn socket_server_send_reply(
    _server: *const u8,
    _reply: *const u8,
) -> c_int { OK }

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
