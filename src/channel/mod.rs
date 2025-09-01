use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;

/// Type of the callback function used for receiving data.
type ChannelCallback = extern "C" fn(*mut c_void, *const c_char, usize);

/// Opaque channel structure exposed over FFI.
#[repr(C)]
pub struct Channel {
    stream: TcpStream,
    runtime: Runtime,
    timeout: Duration,
    callback: Option<(ChannelCallback, *mut c_void)>,
}

/// Open a TCP connection to `addr` and return a new `Channel`.
#[no_mangle]
pub extern "C" fn channel_open(addr: *const c_char) -> *mut Channel {
    if addr.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(addr) };
    let addr_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return std::ptr::null_mut(),
    };
    let stream = match rt.block_on(TcpStream::connect(addr_str)) {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let chan = Channel {
        stream,
        runtime: rt,
        timeout: Duration::from_secs(5),
        callback: None,
    };
    Box::into_raw(Box::new(chan))
}

/// Send `len` bytes from `data` over the channel.  Returns 0 on success.
#[no_mangle]
pub extern "C" fn channel_send(
    chan: *mut Channel,
    data: *const c_char,
    len: usize,
) -> c_int {
    if chan.is_null() || data.is_null() {
        return -1;
    }
    let chan = unsafe { &mut *chan };
    let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
    match chan.runtime.block_on(chan.stream.write_all(slice)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Receive up to `len` bytes into `buf`.  Returns the number of bytes read or -1 on error.
#[no_mangle]
pub extern "C" fn channel_receive(
    chan: *mut Channel,
    buf: *mut c_char,
    len: usize,
) -> isize {
    if chan.is_null() || buf.is_null() {
        return -1;
    }
    let chan = unsafe { &mut *chan };
    let slice = unsafe { std::slice::from_raw_parts_mut(buf as *mut u8, len) };
    match chan.runtime.block_on(chan.stream.read(slice)) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

/// Register a callback that will be invoked with received data when `channel_poll` is called.
#[no_mangle]
pub extern "C" fn channel_set_callback(
    chan: *mut Channel,
    cb: Option<ChannelCallback>,
    userdata: *mut c_void,
) {
    if let Some(chan) = unsafe { chan.as_mut() } {
        chan.callback = cb.map(|c| (c, userdata));
    }
}

/// Poll the channel for new data and invoke the callback if data is available.
#[no_mangle]
pub extern "C" fn channel_poll(chan: *mut Channel) {
    if let Some(chan) = unsafe { chan.as_mut() } {
        if let Some((cb, ud)) = chan.callback {
            let mut buf = [0u8; 1024];
            if let Ok(n) = chan.stream.try_read(&mut buf) {
                if n > 0 {
                    cb(ud, buf.as_ptr() as *const c_char, n);
                }
            }
        }
    }
}

/// Close the channel and free associated resources.
#[no_mangle]
pub extern "C" fn channel_close(chan: *mut Channel) {
    if !chan.is_null() {
        unsafe { drop(Box::from_raw(chan)); }
    }
}
