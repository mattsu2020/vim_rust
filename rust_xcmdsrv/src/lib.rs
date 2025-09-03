use std::ffi::CStr;
use std::io::Write;
use std::net::TcpStream;
use std::os::raw::{c_char, c_int};

/// Send a command to a Vim server over TCP.
///
/// The `cmd` argument is expected to be a C string of the form
/// `"host:port:message"`.  The message is sent as raw bytes to the
/// specified host and port.  On success `0` is returned; on any failure
/// `-1` is returned.
#[no_mangle]
pub extern "C" fn vim_xcmdsrv_send(cmd: *const c_char) -> c_int {
    // Safety: the caller guarantees that `cmd` is a valid C string or NULL.
    if cmd.is_null() {
        return -1;
    }

    let c_slice = unsafe { CStr::from_ptr(cmd) };
    let cmd_str = match c_slice.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match send_over_tcp(cmd_str) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

fn send_over_tcp(spec: &str) -> std::io::Result<()> {
    // Expected format: host:port:message
    let mut parts = spec.splitn(3, ':');
    let host = parts.next().unwrap_or("127.0.0.1");
    let port = parts.next().unwrap_or("0");
    let msg = parts.next().unwrap_or("");

    let addr = format!("{}:{}", host, port);
    let mut stream = TcpStream::connect(addr)?;
    stream.write_all(msg.as_bytes())?;
    Ok(())
}
