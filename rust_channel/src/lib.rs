use std::collections::VecDeque;
use std::ffi::{CStr, c_void};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::runtime::Runtime;

/// Callback type used by the C side.
pub type ChannelCallback = unsafe extern "C" fn(*mut c_void, *const c_char, usize);

/// Channel wraps a TCP connection and stores queued messages until the C side
/// polls for them.  A callback can be registered to be invoked for every queued
/// message.
pub struct Channel {
    rt: Runtime,
    writer: OwnedWriteHalf,
    cb: Arc<Mutex<Option<(ChannelCallback, *mut c_void)>>>,
    queue: Arc<Mutex<VecDeque<Vec<u8>>>>,
}

/// Opens a TCP connection to the given address ("host:port") and returns an
/// opaque pointer to a `Channel` on success. `NULL` is returned on failure.
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
        Ok(stream) => {
            let cb = Arc::new(Mutex::new(None));
            let queue = Arc::new(Mutex::new(VecDeque::new()));
            let (mut reader, writer) = stream.into_split();
            let queue_clone = queue.clone();
            rt.spawn(async move {
                let mut buf = vec![0u8; 1024];
                loop {
                    match reader.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let mut data = vec![0u8; n];
                            data.copy_from_slice(&buf[..n]);
                            queue_clone.lock().unwrap().push_back(data);
                        }
                        Err(_) => break,
                    }
                }
            });
            Box::into_raw(Box::new(Channel { rt, writer, cb, queue }))
        }
        Err(_) => ptr::null_mut(),
    }
}

// Export aliases with rs_ prefix to avoid C symbol clashes
#[no_mangle]
pub extern "C" fn rs_channel_open(addr: *const c_char) -> *mut Channel {
    channel_open(addr)
}

/// Register a callback to be invoked for received messages. Passing a NULL
/// callback unregisters the current one.
#[no_mangle]
pub extern "C" fn channel_set_callback(
    chan: *mut Channel,
    cb: Option<ChannelCallback>,
    userdata: *mut c_void,
) {
    if let Some(chan) = unsafe { chan.as_mut() } {
        let mut lock = chan.cb.lock().unwrap();
        if let Some(cb) = cb {
            *lock = Some((cb, userdata));
        } else {
            *lock = None;
        }
    }
}

/// Poll for queued messages and invoke the registered callback for each.
#[no_mangle]
pub extern "C" fn channel_poll(chan: *mut Channel) {
    if let Some(chan) = unsafe { chan.as_mut() } {
        let cb = { chan.cb.lock().unwrap().clone() };
        if cb.is_none() {
            return;
        }
        let (cb, userdata) = cb.unwrap();
        let mut queue = chan.queue.lock().unwrap();
        while let Some(msg) = queue.pop_front() {
            unsafe { cb(userdata, msg.as_ptr() as *const c_char, msg.len()) };
        }
    }
}

/// Sends `len` bytes from `data` over the channel. Returns 0 on success and -1
/// on failure.
pub extern "C" fn channel_send(chan: *mut Channel, data: *const c_char, len: usize) -> c_int {
    if chan.is_null() || data.is_null() {
        return -1;
    }
    let chan = unsafe { &mut *chan };
    let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
    match chan.rt.block_on(chan.writer.write_all(slice)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn rs_channel_send(chan: *mut Channel, data: *const c_char, len: usize) -> c_int {
    channel_send(chan, data, len)
}

/// Receives at most `len` bytes into `buf`. Returns the number of bytes read or
/// -1 on error.
#[no_mangle]
pub extern "C" fn channel_receive(chan: *mut Channel, buf: *mut c_char, len: usize) -> isize {
    if chan.is_null() || buf.is_null() {
        return -1;
    }
    let chan = unsafe { &mut *chan };
    if let Some(msg) = chan.queue.lock().unwrap().pop_front() {
        let n = std::cmp::min(len, msg.len());
        unsafe { std::ptr::copy_nonoverlapping(msg.as_ptr(), buf as *mut u8, n); }
        n as isize
    } else {
        0
    }
}

/// Closes the channel and frees the underlying resources.
pub extern "C" fn channel_close(chan: *mut Channel) {
    if chan.is_null() {
        return;
    }
    unsafe { drop(Box::from_raw(chan)); }
}

#[no_mangle]
pub extern "C" fn rs_channel_close(chan: *mut Channel) {
    channel_close(chan)
}

