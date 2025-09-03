use std::env;
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;

// Maximum number of bytes we are willing to read or write in one go.  This
// prevents passing a ridiculously large length from C and accidentally
// allocating excessive memory or overflowing usize calculations.
const MAX_IO_SIZE: usize = 10 * 1024 * 1024; // 10MB

use rust_path::normalize_path;

fn c_path_to_normalized(ptr: *const c_char) -> Option<PathBuf> {
    if ptr.is_null() {
        return None;
    }
    let c_path = unsafe { CStr::from_ptr(ptr) };
    let path_str = c_path.to_str().ok()?;
    normalize_path(path_str)
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from(path_str)))
}

/// Read the file at `fname`.
/// Unused parameters mirror the original C API.
#[no_mangle]
pub extern "C" fn readfile(
    fname: *const c_char,
    _sfname: *const c_char,
    _from: isize,
    _lines_to_skip: isize,
    _lines_to_read: isize,
    _eap: *mut c_void,
    _flags: c_int,
) -> c_int {
    let norm = match c_path_to_normalized(fname) {
        Some(p) => p,
        None => return -1,
    };

    match fs::metadata(&norm) {
        Ok(meta) if meta.len() as usize <= MAX_IO_SIZE => (),
        _ => return -1,
    }

    match fs::read(&norm) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Write `len` bytes from `data` to the file at `fname`.
#[no_mangle]
pub extern "C" fn writefile(
    fname: *const c_char,
    data: *const c_char,
    len: usize,
    _flags: c_int,
) -> c_int {
    if data.is_null() {
        return -1;
    }
    let norm = match c_path_to_normalized(fname) {
        Some(p) => p,
        None => return -1,
    };
    if len > MAX_IO_SIZE {
        return -1;
    }
    let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
    match fs::write(&norm, slice) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn rs_findfile(name: *const c_char, path: *const c_char) -> *mut c_char {
    if name.is_null() || path.is_null() {
        return std::ptr::null_mut();
    }
    let cname = unsafe { CStr::from_ptr(name) };
    let cpath = unsafe { CStr::from_ptr(path) };
    let (name_str, path_str) = match (cname.to_str(), cpath.to_str()) {
        (Ok(n), Ok(p)) => (n, p),
        _ => return std::ptr::null_mut(),
    };
    for dir in env::split_paths(path_str) {
        let candidate = dir.join(name_str);
        if candidate.is_file() {
            if let Some(s) = candidate.to_str() {
                return CString::new(s).unwrap().into_raw();
            }
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn rs_read_viminfo(path: *const c_char) -> *mut c_char {
    let norm = match c_path_to_normalized(path) {
        Some(p) => p,
        None => return std::ptr::null_mut(),
    };
    match fs::metadata(&norm) {
        Ok(meta) if meta.len() as usize <= MAX_IO_SIZE => (),
        _ => return std::ptr::null_mut(),
    }
    match fs::read_to_string(&norm) {
        Ok(s) => CString::new(s).unwrap().into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rs_write_viminfo(path: *const c_char, data: *const c_char) -> c_int {
    if data.is_null() {
        return -1;
    }
    let norm = match c_path_to_normalized(path) {
        Some(p) => p,
        None => return -1,
    };
    let content = unsafe { CStr::from_ptr(data) };
    let slice = match content.to_str() {
        Ok(s) => s.as_bytes(),
        Err(_) => return -1,
    };
    if slice.len() > MAX_IO_SIZE {
        return -1;
    }
    match fs::write(&norm, slice) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::fs;

    #[test]
    fn write_then_read() {
        let name = CString::new("./tmp_rust_fileio.txt").unwrap();
        fs::File::create("tmp_rust_fileio.txt").unwrap();
        let data = b"hello rust";
        assert_eq!(
            writefile(name.as_ptr(), data.as_ptr() as *const c_char, data.len(), 0),
            0
        );
        assert_eq!(
            readfile(
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
        fs::File::create(&file_path).unwrap();
        let cpath = CString::new(file_path.to_str().unwrap()).unwrap();
        let data = vec![b'a'; 5 * 1024 * 1024]; // 5MB
        assert_eq!(
            writefile(
                cpath.as_ptr(),
                data.as_ptr() as *const c_char,
                data.len(),
                0
            ),
            0
        );
        assert_eq!(
            readfile(
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
        fs::File::create("tmp_large.txt").unwrap();
        // Intentionally pass a huge length with a tiny buffer. The function
        // should reject this without attempting the write.
        let data = [0u8; 1];
        assert_eq!(
            writefile(
                name.as_ptr(),
                data.as_ptr() as *const c_char,
                MAX_IO_SIZE + 1,
                0
            ),
            -1
        );
        fs::remove_file("tmp_large.txt").unwrap();
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
            readfile(
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

    unsafe fn call_read(path: *const c_char) -> c_int {
        readfile(path, std::ptr::null(), 0, 0, 0, std::ptr::null_mut(), 0)
    }

    unsafe fn call_write(path: *const c_char) -> c_int {
        let buf = [0u8; 1];
        writefile(path, buf.as_ptr() as *const c_char, buf.len(), 0)
    }

    unsafe fn check_validation(f: unsafe fn(*const c_char) -> c_int) {
        assert_eq!(f(std::ptr::null()), -1);

        let invalid = CString::new(vec![0x80, 0x81]).unwrap();
        assert_eq!(f(invalid.as_ptr()), -1);
    }

    unsafe fn check_missing(f: unsafe fn(*const c_char) -> c_int) {
        let dir = tempfile::tempdir().unwrap();
        let missing_path = dir.path().join("no_such_file");
        let missing = CString::new(missing_path.to_str().unwrap()).unwrap();
        assert_eq!(f(missing.as_ptr()), -1);
    }

    unsafe fn check_success(f: unsafe fn(*const c_char) -> c_int) {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let cpath = CString::new(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(f(cpath.as_ptr()), 0);
    }

    #[test]
    fn common_path_validation() {
        unsafe {
            check_success(call_read);
            check_validation(call_read);
            check_missing(call_read);

            check_success(call_write);
            check_validation(call_write);
        }
    }

    #[test]
    fn findfile_finds_file() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("foo.txt");
        fs::write(&file_path, "").unwrap();
        let name = CString::new("foo.txt").unwrap();
        let paths = CString::new(dir.path().to_str().unwrap()).unwrap();
        let res_ptr = rs_findfile(name.as_ptr(), paths.as_ptr());
        assert!(!res_ptr.is_null());
        let res = unsafe { CStr::from_ptr(res_ptr) }
            .to_str()
            .unwrap()
            .to_string();
        unsafe {
            let _ = CString::from_raw(res_ptr);
        }
        assert_eq!(res, file_path.to_str().unwrap());
    }

    #[test]
    fn viminfo_roundtrip() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("viminfo");
        let path = CString::new(file_path.to_str().unwrap()).unwrap();
        let data = CString::new("hello viminfo").unwrap();
        assert_eq!(rs_write_viminfo(path.as_ptr(), data.as_ptr()), 0);
        let read_ptr = rs_read_viminfo(path.as_ptr());
        assert!(!read_ptr.is_null());
        let read = unsafe { CStr::from_ptr(read_ptr) }
            .to_str()
            .unwrap()
            .to_string();
        unsafe {
            let _ = CString::from_raw(read_ptr);
        }
        assert_eq!(read, "hello viminfo");
    }
}
