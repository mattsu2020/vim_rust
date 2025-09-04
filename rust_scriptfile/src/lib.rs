use std::os::raw::c_char;
use rust_fileio::readfile;

/// Attempt to read a Vim script file at `path`.
///
/// This function delegates to the existing `readfile` implementation in the
/// `rust_fileio` crate, passing default parameters.  It returns `true` on
/// success and `false` otherwise.
#[no_mangle]
pub extern "C" fn read_scriptfile_rs(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let result = readfile(path, std::ptr::null(), 0, 0, 0, std::ptr::null_mut(), 0);
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::io::Write;

    #[test]
    fn can_read_scriptfile() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.vim");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all(b"echo test").unwrap();
        }
        let c_path = CString::new(file_path.to_str().unwrap()).unwrap();
        assert!(read_scriptfile_rs(c_path.as_ptr()));
    }
}
