use libc::{c_int, c_void, size_t, ssize_t, EINTR};
use std::io::Error;

/// Read from `fd` into `buf`, retrying on EINTR.
#[no_mangle]
pub extern "C" fn read_eintr(fd: c_int, buf: *mut c_void, bufsize: size_t) -> ssize_t {
    loop {
        let ret = unsafe { libc::read(fd, buf, bufsize) };
        if ret >= 0 {
            return ret;
        }
        let err = Error::last_os_error().raw_os_error().unwrap_or(0);
        if err != EINTR {
            return ret;
        }
    }
}

/// Write the full buffer `buf` to `fd`, retrying on EINTR.
/// Returns the number of bytes written or -1 on error.
#[no_mangle]
pub extern "C" fn write_eintr(fd: c_int, buf: *const c_void, bufsize: size_t) -> ssize_t {
    let mut total: ssize_t = 0;
    while (total as size_t) < bufsize {
        let ptr = unsafe { (buf as *const u8).add(total as usize) };
        let ret = unsafe { libc::write(fd, ptr as *const c_void, bufsize - total as size_t) };
        if ret < 0 {
            let err = Error::last_os_error().raw_os_error().unwrap_or(0);
            if err != EINTR {
                return ret;
            }
        } else {
            total += ret;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use libc::{pipe, close};

    #[test]
    fn roundtrip_pipe() {
        unsafe {
            let mut fds = [0; 2];
            assert_eq!(pipe(fds.as_mut_ptr()), 0);
            let msg = b"hi";
            let w = write_eintr(fds[1], msg.as_ptr() as *const _, msg.len());
            assert_eq!(w, msg.len() as isize);
            let mut buf = [0u8; 2];
            let r = read_eintr(fds[0], buf.as_mut_ptr() as *mut _, buf.len());
            assert_eq!(r, msg.len() as isize);
            assert_eq!(&buf, msg);
            close(fds[0]);
            close(fds[1]);
        }
    }
}
