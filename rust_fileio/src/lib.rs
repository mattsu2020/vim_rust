use std::ffi::CStr;
use std::fs;
use std::os::raw::{c_char, c_int, c_void};

// Maximum number of bytes we are willing to read or write in one go.  This
// prevents passing a ridiculously large length from C and accidentally
// allocating excessive memory or overflowing usize calculations.
const MAX_IO_SIZE: usize = 10 * 1024 * 1024; // 10MB

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
    let norm = normalize_path(path_str).unwrap_or_else(|| path_str.to_string());

    match fs::metadata(&norm) {
        Ok(meta) if meta.len() as usize <= MAX_IO_SIZE => (),
        _ => return -1,
    }

    match fs_err::read(&norm) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("read error: {err}");
            -1
        }
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
    let norm = normalize_path(path_str).unwrap_or_else(|| path_str.to_string());
    if len > MAX_IO_SIZE {
        return -1;
    }
    let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
    match fs_err::write(&norm, slice) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("write error: {err}");
            -1
        }
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

    #[test]
    fn write_too_large() {
        use std::ffi::CString;
        let name = CString::new("./tmp_large.txt").unwrap();
        // Intentionally pass a huge length with a tiny buffer. The function
        // should reject this without attempting the write.
        let data = [0u8; 1];
        assert_eq!(
            rs_writefile(
                name.as_ptr(),
                data.as_ptr() as *const c_char,
                MAX_IO_SIZE + 1,
                0
            ),
            -1
        );
    }

    #[test]
    fn read_too_large() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("huge.bin");
        {
            use std::io::Write;
            let mut f = fs::File::create(&file_path).unwrap();
            f.write_all(&vec![0u8; MAX_IO_SIZE + 1]).unwrap();
        }
        let cpath = CString::new(file_path.to_str().unwrap()).unwrap();
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
            -1
        );
    }
}
