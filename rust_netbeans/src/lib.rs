use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

use rust_channel::{channel_close, channel_open, channel_receive, channel_send, Channel};
use rust_nbdebug::nbdbg;

/// Safe wrapper around the low-level `Channel` to communicate using the
/// NetBeans protocol.
pub struct NetBeansClient {
    chan: *mut Channel,
}

impl NetBeansClient {
    /// Connect to a NetBeans server at the given address (host:port).
    pub fn connect(addr: &str) -> Option<Self> {
        let caddr = CString::new(addr).ok()?;
        let chan = channel_open(caddr.as_ptr());
        if chan.is_null() {
            None
        } else {
            Some(Self { chan })
        }
    }

    /// Send a raw protocol command.
    pub fn send(&self, cmd: &str) -> bool {
        nbdbg(&format!("SEND: {}", cmd));
        let c = CString::new(cmd).unwrap();
        channel_send(self.chan, c.as_ptr(), c.as_bytes().len()) == 0
    }

    /// Receive a message from the server, if any.
    pub fn receive(&self) -> Option<String> {
        let mut buf = vec![0u8; 4096];
        let n = channel_receive(self.chan, buf.as_mut_ptr() as *mut c_char, buf.len());
        if n > 0 {
            let msg = String::from_utf8_lossy(&buf[..n as usize]).into_owned();
            nbdbg(&format!("RECV: {}", msg));
            Some(msg)
        } else {
            None
        }
    }

    /// Close the connection.
    pub fn close(self) {
        channel_close(self.chan)
    }
}

#[no_mangle]
pub extern "C" fn rs_netbeans_connect(addr: *const c_char) -> *mut NetBeansClient {
    if addr.is_null() {
        return ptr::null_mut();
    }
    let addr_str = unsafe { CStr::from_ptr(addr).to_string_lossy().to_string() };
    match NetBeansClient::connect(&addr_str) {
        Some(c) => Box::into_raw(Box::new(c)),
        None => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rs_netbeans_send(client: *mut NetBeansClient, msg: *const c_char) -> c_int {
    if client.is_null() || msg.is_null() {
        return -1;
    }
    let client = unsafe { &mut *client };
    let msg_str = unsafe { CStr::from_ptr(msg).to_string_lossy().to_string() };
    if client.send(&msg_str) {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn rs_netbeans_receive(
    client: *mut NetBeansClient,
    buf: *mut c_char,
    len: usize,
) -> isize {
    if client.is_null() || buf.is_null() {
        return -1;
    }
    let client = unsafe { &mut *client };
    if let Some(msg) = client.receive() {
        let bytes = msg.as_bytes();
        let n = len.min(bytes.len());
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf as *mut u8, n);
        }
        n as isize
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn rs_netbeans_close(client: *mut NetBeansClient) {
    if client.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(client).close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[test]
    fn basic_comm() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let n = sock.read(&mut buf).unwrap();
            sock.write_all(&buf[..n]).unwrap();
        });
        let caddr = CString::new(format!("{}", addr)).unwrap();
        let client_ptr = rs_netbeans_connect(caddr.as_ptr());
        assert!(!client_ptr.is_null());
        let msg = CString::new("ping").unwrap();
        assert_eq!(rs_netbeans_send(client_ptr, msg.as_ptr()), 0);
        std::thread::sleep(std::time::Duration::from_millis(100));
        let mut buf = [0i8; 1024];
        let n = rs_netbeans_receive(client_ptr, buf.as_mut_ptr(), 1024);
        assert!(n > 0);
        rs_netbeans_close(client_ptr);
    }
}
