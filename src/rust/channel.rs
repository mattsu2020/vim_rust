use std::os::raw::{c_int, c_void};
use std::slice;

#[cfg(unix)]
use std::os::unix::io::{FromRawFd, RawFd};
#[cfg(windows)]
use std::os::windows::io::{FromRawSocket, RawSocket};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[cfg(unix)]
fn raw_socket(fd: c_int) -> RawFd {
    fd as RawFd
}

#[cfg(windows)]
fn raw_socket(fd: c_int) -> RawSocket {
    fd as RawSocket
}

async fn async_read(fd: c_int, buf: &mut [u8]) -> std::io::Result<usize> {
    #[cfg(unix)]
    let std_stream = unsafe { std::net::TcpStream::from_raw_fd(raw_socket(fd)) };
    #[cfg(windows)]
    let std_stream = unsafe { std::net::TcpStream::from_raw_socket(raw_socket(fd)) };
    std_stream.set_nonblocking(true)?;
    let mut stream = tokio::net::TcpStream::from_std(std_stream)?;
    let res = stream.read(buf).await;
    std::mem::forget(stream);
    res
}

async fn async_write(fd: c_int, buf: &[u8]) -> std::io::Result<usize> {
    #[cfg(unix)]
    let std_stream = unsafe { std::net::TcpStream::from_raw_fd(raw_socket(fd)) };
    #[cfg(windows)]
    let std_stream = unsafe { std::net::TcpStream::from_raw_socket(raw_socket(fd)) };
    std_stream.set_nonblocking(true)?;
    let mut stream = tokio::net::TcpStream::from_std(std_stream)?;
    let res = stream.write(buf).await;
    std::mem::forget(stream);
    res
}

#[no_mangle]
pub extern "C" fn channel_read_rs(fd: c_int, buf: *mut c_void, len: usize) -> isize {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };
    let slice = unsafe { slice::from_raw_parts_mut(buf as *mut u8, len) };
    match runtime.block_on(async_read(fd, slice)) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn channel_write_rs(fd: c_int, buf: *const c_void, len: usize) -> isize {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };
    let slice = unsafe { slice::from_raw_parts(buf as *const u8, len) };
    match runtime.block_on(async_write(fd, slice)) {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn channel_close_rs(fd: c_int) -> c_int {
    #[cfg(unix)]
    unsafe {
        libc::close(fd)
    }
    #[cfg(windows)]
    unsafe {
        libc::closesocket(fd as libc::SOCKET)
    }
}
