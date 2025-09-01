use std::ffi::CStr;
use std::fs;
use std::os::raw::{c_char, c_int};

/// Read the file at `path`.  Returns 0 on success and -1 on error.
#[no_mangle]
pub extern "C" fn rs_readfile(path: *const c_char) -> c_int {
    if path.is_null() {
        return -1;
    }
    let c_path = unsafe { CStr::from_ptr(path) };
    match c_path.to_str() {
        Ok(p) => match fs::read(p) {
            Ok(_) => 0,
            Err(_) => -1,
        },
        Err(_) => -1,
    }
}

/// Write `len` bytes from `data` to the file at `path`.
/// Returns 0 on success and -1 on failure.
#[no_mangle]
pub extern "C" fn rs_writefile(path: *const c_char, data: *const c_char, len: usize) -> c_int {
    if path.is_null() || data.is_null() {
        return -1;
    }
    let c_path = unsafe { CStr::from_ptr(path) };
    let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
    match c_path.to_str() {
        Ok(p) => match fs::write(p, slice) {
            Ok(_) => 0,
            Err(_) => -1,
        },
        Err(_) => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::fs;

    #[test]
    fn write_then_read() {
        let name = CString::new("tmp_rust_fileio.txt").unwrap();
        let data = b"hello rust";
        assert_eq!(
            rs_writefile(name.as_ptr(), data.as_ptr() as *const c_char, data.len()),
            0
        );
        assert_eq!(rs_readfile(name.as_ptr()), 0);
        let content = fs::read_to_string("tmp_rust_fileio.txt").unwrap();
        assert_eq!(content, "hello rust");
        fs::remove_file("tmp_rust_fileio.txt").unwrap();
    }
}
