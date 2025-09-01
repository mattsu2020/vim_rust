use libc::{c_char, c_int, size_t};

/// Simplified representation of Vim's memfile_T.
/// This structure keeps all lines in memory without swap management.
#[repr(C)]
pub struct MemFile {
    lines: Vec<Vec<u8>>, // stored as UTF-8 bytes
}

impl MemFile {
    fn new() -> Self {
        Self { lines: Vec::new() }
    }

    fn append(&mut self, line: &[u8]) {
        self.lines.push(line.to_vec());
    }
}

/// Open a memory file. The parameters are kept for FFI compatibility but
/// are ignored in this simplified implementation.
#[no_mangle]
pub extern "C" fn mf_open(_fname: *const c_char, _flags: c_int) -> *mut MemFile {
    Box::into_raw(Box::new(MemFile::new()))
}

/// Append a line to the memory file.
#[no_mangle]
pub extern "C" fn ml_append(mf: *mut MemFile, line: *const c_char, len: size_t) -> c_int {
    if mf.is_null() || line.is_null() {
        return -1;
    }
    let slice = unsafe { std::slice::from_raw_parts(line as *const u8, len as usize) };
    let mf = unsafe { &mut *mf };
    mf.append(slice);
    0
}

/// Close the memory file and free its resources.
#[no_mangle]
pub extern "C" fn mf_close(mf: *mut MemFile) {
    if !mf.is_null() {
        unsafe { drop(Box::from_raw(mf)); }
    }
}

/// Get the number of lines currently stored. Exposed for testing.
#[no_mangle]
pub extern "C" fn ml_line_count(mf: *const MemFile) -> size_t {
    if mf.is_null() { return 0; }
    let mf = unsafe { &*mf };
    mf.lines.len() as size_t
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn append_and_count() {
        let mf = mf_open(ptr::null(), 0);
        let line = CString::new("hello").unwrap();
        ml_append(mf, line.as_ptr(), 5);
        let count = ml_line_count(mf);
        mf_close(mf);
        assert_eq!(count, 1);
    }

    #[test]
    fn append_many_lines() {
        let mf = mf_open(ptr::null(), 0);
        for i in 0..1000 {
            let s = format!("line{}", i);
            let c = CString::new(s).unwrap();
            ml_append(mf, c.as_ptr(), c.as_bytes().len());
        }
        let count = ml_line_count(mf);
        mf_close(mf);
        assert_eq!(count, 1000);
    }
}
