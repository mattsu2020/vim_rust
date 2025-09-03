use std::ffi::CStr;
use std::fs;
use std::os::raw::{c_char, c_int, c_void};

use rust_path::normalize_path;

/// Read the file at `fname`.
/// Unused parameters mirror the original C API.
#[no_mangle]
pub extern "C" fn rs_readfile(
    fname: *const c_char,
    _sfname: *const c_char,
    _from: isize,
    _lines_to_skip: isize,
    _lines_to_read: isize,
    _eap: *mut c_void,
    _flags: c_int,
) -> c_int {
    if fname.is_null() {
        return -1;
    }
    let c_path = unsafe { CStr::from_ptr(fname) };
    let path_str = match c_path.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let norm = match normalize_path(path_str) {
        Some(p) => p,
        None => return -1,
    };
    match fs::read(&norm) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Write `len` bytes from `data` to the file at `fname`.
#[no_mangle]
pub extern "C" fn rs_writefile(
    fname: *const c_char,
    data: *const c_char,
    len: usize,
    _flags: c_int,
) -> c_int {
    if fname.is_null() || data.is_null() {
        return -1;
    }
    let c_path = unsafe { CStr::from_ptr(fname) };
    let path_str = match c_path.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let norm = match normalize_path(path_str) {
        Some(p) => p,
        None => return -1,
    };
    let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
    match fs::write(&norm, slice) {
        Ok(_) => 0,
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
        let name = CString::new("./tmp_rust_fileio.txt").unwrap();
        let data = b"hello rust";
        assert_eq!(
            rs_writefile(name.as_ptr(), data.as_ptr() as *const c_char, data.len(), 0),
            0
        );
        assert_eq!(
            rs_readfile(
                name.as_ptr(),
                std::ptr::null(),
                0,
                0,
                0,
                std::ptr::null_mut(),
                0
            ),
            0
        );
        let content = fs::read_to_string("tmp_rust_fileio.txt").unwrap();
        assert_eq!(content, "hello rust");
        fs::remove_file("tmp_rust_fileio.txt").unwrap();
    }

    #[test]
    fn large_file() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.bin");
        let cpath = CString::new(file_path.to_str().unwrap()).unwrap();
        let data = vec![b'a'; 5 * 1024 * 1024]; // 5MB
        assert_eq!(
            rs_writefile(cpath.as_ptr(), data.as_ptr() as *const c_char, data.len(), 0),
            0
        );
        assert_eq!(
            rs_readfile(
                cpath.as_ptr(),
                std::ptr::null(),
                0,
                0,
                0,
                std::ptr::null_mut(),
                0
            ),
            0
        );
        let metadata = fs::metadata(file_path).unwrap();
        assert_eq!(metadata.len(), data.len() as u64);
    }
}
