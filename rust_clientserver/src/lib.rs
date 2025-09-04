use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
use std::net::SocketAddr;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::fs;

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

static SERVER_STATE: Lazy<Mutex<ServerState>> = Lazy::new(|| Mutex::new(ServerState::default()));

#[derive(Default)]
struct ServerState {
    running: bool,
    name: Option<String>,
    socket_path: Option<PathBuf>,
}

const OK: c_int = 1;
const FAIL: c_int = 0;

#[no_mangle]
pub extern "C" fn socket_server_init(_servername: *const u8) -> c_int {
    let name = unsafe {
        if _servername.is_null() { "VIM".to_string() } else {
            CStr::from_ptr(_servername as *const c_char)
                .to_string_lossy().into_owned()
        }
    };
    let mut st = SERVER_STATE.lock().unwrap();
    if st.running {
        return OK;
    }
    st.name = Some(name.clone());

    // Derive a socket path under /tmp
    #[cfg(unix)]
    {
        let dir = Path::new("/tmp").join("vim-rust-server");
        let _ = fs::create_dir_all(&dir);
        let sock = dir.join(format!("{}.sock", name));
        // Cleanup stale socket
        if sock.exists() { let _ = fs::remove_file(&sock); }
        st.socket_path = Some(sock.clone());

        // Spawn background runtime and listener
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("rt");
            rt.block_on(async move {
                if let Ok(listener) = UnixListener::bind(&sock) {
                    loop {
                        match listener.accept().await {
                            Ok((mut stream, _addr)) => {
                                tokio::spawn(async move {
                                    let mut buf = Vec::with_capacity(1024);
                                    let mut tmp = [0u8; 1024];
                                    // Simple protocol: first byte tag ('E' or 'K'), then nul-terminated string
                                    if let Ok(n) = stream.read(&mut tmp).await {
                                        if n > 0 {
                                            buf.extend_from_slice(&tmp[..n]);
                                            let tag = buf.get(0).copied().unwrap_or(b'K');
                                            // extract c-string after tag
                                            let payload = if buf.len() > 1 {
                                                &buf[1..]
                                            } else { &[][..] };
                                            // ensure terminated
                                            let end = payload.iter().position(|&c| c == 0).unwrap_or(payload.len());
                                            let s = String::from_utf8_lossy(&payload[..end]).into_owned();
                                            if tag == b'E' {
                                                // Evaluate via rust_eval's eval_to_string()
                                                let c = CString::new(s).unwrap();
                                                unsafe {
                                                    extern "C" { fn eval_to_string(expr:*const u8, sandbox: c_int, remove: c_int) -> *mut u8; }
                                                    let ptr = eval_to_string(c.as_ptr() as *const u8, 0, 0);
                                                    if !ptr.is_null() {
                                                        // send back nul-terminated bytes prefixed with 'R'
                                                        let cstr = CStr::from_ptr(ptr as *const c_char);
                                                        let mut out = Vec::with_capacity(1 + cstr.to_bytes().len() + 1);
                                                        out.push(b'R');
                                                        out.extend_from_slice(cstr.to_bytes());
                                                        out.push(0);
                                                        let _ = stream.write_all(&out).await;
                                                    } else {
                                                        let _ = stream.write_all(b"R\0").await;
                                                    }
                                                }
                                            } else {
                                                // Treat as keys: acknowledge
                                                let _ = stream.write_all(b"O\0").await;
                                            }
                                        }
                                    }
                                });
                            }
                            Err(_) => break,
                        }
                    }
                }
            });
        });
    }

    st.running = true;
    OK
}

#[no_mangle]
pub extern "C" fn socket_server_uninit() {
    let mut st = SERVER_STATE.lock().unwrap();
    if let Some(p) = &st.socket_path { let _ = fs::remove_file(p); }
    st.running = false;
}

#[no_mangle]
pub extern "C" fn socket_server_valid() -> c_int {
    if SERVER_STATE.lock().unwrap().running { OK } else { FAIL }
}

#[no_mangle]
pub extern "C" fn socket_server_waiting_accept() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn socket_server_list_sockets() -> *mut u8 {
    let st = SERVER_STATE.lock().unwrap();
    let name = st.name.clone().unwrap_or_else(|| "VIM".to_string());
    let s = CString::new(name).unwrap();
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
    let name = unsafe {
        if _servername.is_null() { "VIM".to_string() } else {
            CStr::from_ptr(_servername as *const c_char)
                .to_string_lossy().into_owned()
        }
    };

    let st = SERVER_STATE.lock().unwrap();
    if !st.running { drop(st); return FAIL; }
    let sock = match &st.socket_path { Some(p) => p.clone(), None => { drop(st); return FAIL; } };
    drop(st);

    // Detect whether 5th argument is a pointer to expr (remote_expr) or a flag (<=1)
    let is_expr_ptr = _expr_flag_or_ptr > 4096;
    let (tag, payload) = unsafe {
        if is_expr_ptr {
            (b'E', _expr_flag_or_ptr as usize as *const u8)
        } else {
            (b'K', _keys)
        }
    };

    #[cfg(unix)]
    {
        // Connect and send request
        let rt = tokio::runtime::Runtime::new().unwrap();
        let rc = rt.block_on(async move {
            match UnixStream::connect(&sock).await {
                Ok(mut stream) => {
                    // Compose: tag + c-string
                    let cstr = if payload.is_null() { CString::new("").unwrap() } else { CStr::from_ptr(payload as *const c_char).to_owned() };
                    let mut buf = Vec::with_capacity(1 + cstr.as_bytes().len() + 1);
                    buf.push(tag);
                    buf.extend_from_slice(cstr.as_bytes());
                    buf.push(0);
                    if stream.write_all(&buf).await.is_err() { return FAIL; }
                    // Read reply if expecting immediate
                    let mut rbuf = [0u8; 4096];
                    match stream.read(&mut rbuf).await {
                        Ok(n) if n > 0 => {
                            if rbuf[0] == b'R' {
                                // Expression result
                                if !result.is_null() {
                                    let s = CStr::from_ptr(&rbuf[1] as *const u8 as *const c_char).to_owned();
                                    unsafe { *result = s.into_raw() as *mut u8; }
                                }
                                OK
                            } else {
                                // OK/other
                                OK
                            }
                        }
                        _ => OK,
                    }
                }
                Err(_) => FAIL,
            }
        });
        return rc;
    }

    // Non-unix fallback: not implemented
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
