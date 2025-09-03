use libc::{c_int, c_void, size_t, EINTR};

/// Write a buffer to a file descriptor, retrying on EINTR.
#[no_mangle]
pub extern "C" fn rs_buf_write(fd: c_int, buf: *const c_void, len: c_int) -> c_int {
    if fd < 0 || buf.is_null() || len < 0 {
        return -1;
    }
    let mut written: c_int = 0;
    while written < len {
        let ptr = unsafe { (buf as *const u8).add(written as usize) } as *const c_void;
        let res = unsafe { libc::write(fd, ptr, (len - written) as size_t) };
        if res < 0 {
            let err = std::io::Error::last_os_error().raw_os_error().unwrap_or(-1);
            if err == EINTR {
                continue;
            }
            return res as c_int;
        }
        written += res as c_int;
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, read_to_string};
    use std::os::fd::AsRawFd;
    use libc::c_void;

    #[test]
    fn write_and_read_back() {
        let mut path = std::env::temp_dir();
        path.push("rs_bufwrite_test.txt");
        let file = File::create(&path).unwrap();
        let fd = file.as_raw_fd();
        let data = b"hello";
        let res = rs_buf_write(fd, data.as_ptr() as *const c_void, data.len() as c_int);
        assert_eq!(res, data.len() as c_int);
        drop(file);
        let contents = read_to_string(&path).unwrap();
        assert_eq!(contents, "hello");
        let _ = std::fs::remove_file(path);
    }
}
