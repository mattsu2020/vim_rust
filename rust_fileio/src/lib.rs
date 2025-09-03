use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;
use tokio::fs;

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
    normalize_path(path_str).map(PathBuf::from)
}

/// Read the file at `fname`.
/// Unused parameters mirror the original C API.
async fn rs_readfile(
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

    match fs::metadata(&norm).await {
        Ok(meta) if meta.len() as usize <= MAX_IO_SIZE => (),
        _ => return -1,
    }

    match fs::read(&norm).await {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Write `len` bytes from `data` to the file at `fname`.
async fn rs_writefile(
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
    match fs::write(&norm, slice).await {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

pub mod ffi {
    use super::*;
    use tokio::runtime::Handle;

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
        Handle::current().block_on(super::rs_readfile(
            fname,
            _sfname,
            _from,
            _lines_to_skip,
            _lines_to_read,
            _eap,
            _flags,
        ))
    }

    #[no_mangle]
    pub extern "C" fn rs_writefile(
        fname: *const c_char,
        data: *const c_char,
        len: usize,
        _flags: c_int,
    ) -> c_int {
        Handle::current().block_on(super::rs_writefile(
            fname, data, len, _flags,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::fs;

    #[tokio::test]
    async fn write_then_read() {
        let name = CString::new("./tmp_rust_fileio.txt").unwrap();
        fs::File::create("tmp_rust_fileio.txt").unwrap();
        let data = b"hello rust";
        assert_eq!(
            rs_writefile(name.as_ptr(), data.as_ptr() as *const c_char, data.len(), 0).await,
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
            ).await,
            0
        );
        let content = fs::read_to_string("tmp_rust_fileio.txt").unwrap();
        assert_eq!(content, "hello rust");
        fs::remove_file("tmp_rust_fileio.txt").unwrap();
    }

    #[tokio::test]
    async fn large_file() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.bin");
        fs::File::create(&file_path).unwrap();
        let cpath = CString::new(file_path.to_str().unwrap()).unwrap();
        let data = vec![b'a'; 5 * 1024 * 1024]; // 5MB
        assert_eq!(
            rs_writefile(
                cpath.as_ptr(),
                data.as_ptr() as *const c_char,
                data.len(),
                0
            ).await,
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
            ).await,
            0
        );
        let metadata = fs::metadata(file_path).unwrap();
        assert_eq!(metadata.len(), data.len() as u64);
    }

    #[tokio::test]
    async fn write_too_large() {
        let name = CString::new("./tmp_large.txt").unwrap();
        fs::File::create("tmp_large.txt").unwrap();
        // Intentionally pass a huge length with a tiny buffer. The function
        // should reject this without attempting the write.
        let data = [0u8; 1];
        assert_eq!(
            rs_writefile(
                name.as_ptr(),
                data.as_ptr() as *const c_char,
                MAX_IO_SIZE + 1,
                0
            ).await,
            -1
        );
        fs::remove_file("tmp_large.txt").unwrap();
    }

    #[tokio::test]
    async fn read_too_large() {
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
            ).await,
            -1
        );
    }

    async fn call_read(path: *const c_char) -> c_int {
        rs_readfile(path, std::ptr::null(), 0, 0, 0, std::ptr::null_mut(), 0).await
    }

    async fn call_write(path: *const c_char) -> c_int {
        let buf = [0u8; 1];
        rs_writefile(path, buf.as_ptr() as *const c_char, buf.len(), 0).await
    }

    async fn check_validation<F, Fut>(f: F)
    where
        F: Fn(*const c_char) -> Fut,
        Fut: std::future::Future<Output = c_int>,
    {
        assert_eq!(f(std::ptr::null()).await, -1);

        let invalid = CString::new(vec![0x80, 0x81]).unwrap();
        assert_eq!(f(invalid.as_ptr()).await, -1);

        let missing = CString::new("./no_such_file").unwrap();
        assert_eq!(f(missing.as_ptr()).await, -1);
    }

    async fn check_success<F, Fut>(f: F)
    where
        F: Fn(*const c_char) -> Fut,
        Fut: std::future::Future<Output = c_int>,
    {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let cpath = CString::new(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(f(cpath.as_ptr()).await, 0);
    }

    #[tokio::test]
    async fn common_path_validation() {
        check_success(call_read).await;
        check_validation(call_read).await;

        check_success(call_write).await;
        check_validation(call_write).await;
    }
}
