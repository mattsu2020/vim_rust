use libc::{c_char, c_int, size_t};
use std::ffi::{CStr};
use std::fs::File;
use memmap2::Mmap;
use std::ptr;
use memchr::memchr;
use memchr::memmem;

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

/// File-backed buffer using a memory mapped file with line offsets.
pub struct FileBuffer {
    map: Mmap,
    line_offsets: Vec<usize>,
}

impl FileBuffer {
    fn open(path: &CStr) -> std::io::Result<Self> {
        let f = File::open(path.to_string_lossy().as_ref())?;
        let map = unsafe { Mmap::map(&f)? };
        let mut offsets = vec![0];
        for (i, b) in map.iter().enumerate() {
            if *b == b'\n' {
                offsets.push(i + 1);
            }
        }
        Ok(Self { map, line_offsets: offsets })
    }

    fn line(&self, idx: usize) -> Option<&[u8]> {
        if idx >= self.line_offsets.len() {
            return None;
        }
        let start = self.line_offsets[idx];
        let end = if idx + 1 < self.line_offsets.len() {
            self.line_offsets[idx + 1] - 1
        } else {
            self.map.len()
        };
        Some(&self.map[start..end])
    }

    fn search(&self, pat: &[u8]) -> Option<usize> {
        memmem::find(&self.map, pat)
    }
}

/// Open a file-backed buffer.
#[no_mangle]
pub extern "C" fn fb_open(path: *const c_char) -> *mut FileBuffer {
    if path.is_null() {
        return ptr::null_mut();
    }
    let c_path = unsafe { CStr::from_ptr(path) };
    match FileBuffer::open(c_path) {
        Ok(fb) => Box::into_raw(Box::new(fb)),
        Err(_) => ptr::null_mut(),
    }
}

/// Close a file-backed buffer.
#[no_mangle]
pub extern "C" fn fb_close(fb: *mut FileBuffer) {
    if !fb.is_null() {
        unsafe { drop(Box::from_raw(fb)); }
    }
}

/// Get a line from a file-backed buffer.
#[no_mangle]
pub extern "C" fn fb_get_line(
    fb: *const FileBuffer,
    idx: size_t,
    len: *mut size_t,
) -> *const c_char {
    if fb.is_null() {
        return ptr::null();
    }
    let fb_ref = unsafe { &*fb };
    if let Some(line) = fb_ref.line(idx as usize) {
        if !len.is_null() {
            unsafe { *len = line.len() as size_t; }
        }
        line.as_ptr() as *const c_char
    } else {
        if !len.is_null() {
            unsafe { *len = 0; }
        }
        ptr::null()
    }
}

/// Search for a pattern in a file-backed buffer. Returns -1 if not found.
#[no_mangle]
pub extern "C" fn fb_search(
    fb: *const FileBuffer,
    pat: *const c_char,
    pat_len: size_t,
) -> isize {
    if fb.is_null() || pat.is_null() {
        return -1;
    }
    let fb_ref = unsafe { &*fb };
    let pattern = unsafe { std::slice::from_raw_parts(pat as *const u8, pat_len as usize) };
    match fb_ref.search(pattern) {
        Some(pos) => pos as isize,
        None => -1,
    }
}

/// A safe wrapper around memchr exposed for C callers.
#[no_mangle]
pub extern "C" fn rs_memchr(
    s: *const c_char,
    c: c_int,
    n: size_t,
) -> *const c_char {
    if s.is_null() {
        return ptr::null();
    }
    let slice = unsafe { std::slice::from_raw_parts(s as *const u8, n as usize) };
    if let Some(pos) = memchr(c as u8, slice) {
        unsafe { s.add(pos) }
    } else {
        ptr::null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;
    use std::fs::File;

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

    #[test]
    fn file_buffer_basic() {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let path = dir.join("fb_test.txt");
        {
            let mut f = File::create(&path).unwrap();
            writeln!(f, "hello").unwrap();
            writeln!(f, "world").unwrap();
        }
        let c_path = CString::new(path.to_str().unwrap()).unwrap();
        let fb = fb_open(c_path.as_ptr());
        assert!(!fb.is_null());
        let mut len: size_t = 0;
        let line = fb_get_line(fb, 0, &mut len as *mut size_t);
        let slice = unsafe { std::slice::from_raw_parts(line as *const u8, len as usize) };
        assert_eq!(slice, b"hello");
        assert_eq!(fb_search(fb, b"world".as_ptr() as *const c_char, 5), 6);
        fb_close(fb);
    }
}
