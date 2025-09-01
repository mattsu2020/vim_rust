use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;

/// Simple Channel structure wrapping a TCP connection. This is a very small
/// subset of Vim's channel.c functionality and is meant as a starting point for
/// rewriting the socket/event loop handling in Rust.
pub struct Channel {
    rt: Runtime,
    stream: TcpStream,
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
        Ok(stream) => Box::into_raw(Box::new(Channel { rt, stream })),
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
    match chan.rt.block_on(chan.stream.write_all(slice)) {
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
    match chan.rt.block_on(chan.stream.read(slice)) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
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
