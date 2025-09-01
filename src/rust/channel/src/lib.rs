use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use std::sync::Arc;

/// Simple Channel structure wrapping a TCP connection. This is a very small
/// subset of Vim's channel.c functionality and is meant as a starting point for
/// rewriting the socket/event loop handling in Rust.
pub struct Channel {
    rt: Runtime,
    stream: Arc<Mutex<TcpStream>>,
}

/// Opens a TCP connection to the given address ("host:port") and returns an
/// opaque pointer to a `Channel` on success.  `NULL` is returned on failure.
#[no_mangle]
pub extern "C" fn channel_open(addr: *const c_char) -> *mut Channel {
    if addr.is_null() {
        return ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(addr) };
    let addr_str = match c_str.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    match rt.block_on(TcpStream::connect(addr_str)) {
        Ok(stream) => Box::into_raw(Box::new(Channel { rt, stream: Arc::new(Mutex::new(stream)) })),
        Err(_) => ptr::null_mut(),
    }
}

/// Sends `len` bytes from `data` over the channel.  Returns 0 on success and -1
/// on failure.
#[no_mangle]
pub extern "C" fn channel_send(chan: *mut Channel, data: *const c_char, len: usize) -> c_int {
    if chan.is_null() || data.is_null() {
        return -1;
    }
    let chan = unsafe { &mut *chan };
    let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
    let stream = chan.stream.clone();
    match chan.rt.block_on(async move { stream.lock().await.write_all(slice).await }) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Receives at most `len` bytes into `buf`.  Returns the number of bytes read or
/// -1 on error.
#[no_mangle]
pub extern "C" fn channel_receive(chan: *mut Channel, buf: *mut c_char, len: usize) -> isize {
    if chan.is_null() || buf.is_null() {
        return -1;
    }
    let chan = unsafe { &mut *chan };
    let slice = unsafe { std::slice::from_raw_parts_mut(buf as *mut u8, len) };
    let stream = chan.stream.clone();
    match chan.rt.block_on(async move { stream.lock().await.read(slice).await }) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

/// Starts an asynchronous read loop on the channel. For every chunk of data
/// read the provided callback `cb` is invoked with the bytes read and the
/// opaque `user` pointer. The loop runs on the internal Tokio runtime and
/// terminates when the stream closes or an error occurs.
/// Returns 0 on success and -1 on failure.
#[no_mangle]
pub extern "C" fn channel_run(
    chan: *mut Channel,
    cb: extern "C" fn(*const c_char, usize, *mut c_void),
    user: *mut c_void,
) -> c_int {
    if chan.is_null() {
        return -1;
    }
    let chan = unsafe { &mut *chan };
    let stream = chan.stream.clone();
    let user_ptr = user as usize;
    chan.rt.spawn(async move {
        let stream = stream;
        let mut buf = vec![0u8; 1024];
        loop {
            let n = {
                let mut locked = stream.lock().await;
                match locked.read(&mut buf).await {
                    Ok(n) => n,
                    Err(_) => break,
                }
            };
            if n == 0 {
                break;
            }
            cb(buf.as_ptr() as *const c_char, n, user_ptr as *mut c_void);
        }
    });
    0
}

/// Closes the channel and frees the underlying resources.
#[no_mangle]
pub extern "C" fn channel_close(chan: *mut Channel) {
    if chan.is_null() {
        return;
    }
    // Dropping the Box will drop the runtime and stream.
    unsafe { drop(Box::from_raw(chan)); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::sync::mpsc::{channel, Sender, Receiver};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn run_callback_receives_data() {
        // Set up a simple TCP server that sends one message and closes.
        let rt = Runtime::new().unwrap();
        let listener = rt.block_on(tokio::net::TcpListener::bind("127.0.0.1:0")).unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = rt.handle().clone();
        thread::spawn(move || {
            handle.block_on(async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                socket.write_all(b"ping").await.unwrap();
            });
        });

        // Open the channel via the FFI API.
        let addr_c = CString::new(addr.to_string()).unwrap();
        let chan = channel_open(addr_c.as_ptr());
        assert!(!chan.is_null());

        // Prepare a callback that sends received data through an mpsc channel.
        extern "C" fn cb(data: *const c_char, len: usize, tx: *mut c_void) {
            let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
            let tx = unsafe { &*(tx as *const Sender<Vec<u8>>) };
            tx.send(slice.to_vec()).unwrap();
        }

        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();
        let tx_box = Box::new(tx);
        let tx_ptr = Box::into_raw(tx_box) as *mut c_void;
        assert_eq!(channel_run(chan, cb, tx_ptr), 0);

        // Wait for the callback to run and verify the message.
        let got = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(got, b"ping".to_vec());

        // Clean up leaked box and channel.
        unsafe { drop(Box::from_raw(tx_ptr as *mut Sender<Vec<u8>>)); };
        channel_close(chan);
    }
}
