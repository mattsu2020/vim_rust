use ropey::{Rope, RopeSlice};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Maximum length of a single line allowed in the memline buffer.  This keeps
// the C side from accidentally passing an unbounded string and exhausting
// memory or overflowing internal calculations.
const MAX_LINE_LEN: usize = 1 << 20; // 1MB per line

/// Simple in-memory representation of lines in a buffer.
#[derive(Default)]
pub struct MemBuffer {
    lines: Rope,
    // Workspace for exposing lines to C as modifiable C strings.
    // Keyed by lnum. Stored with a trailing NUL.
    workspace: HashMap<usize, Vec<u8>>,
}

impl MemBuffer {
    pub fn new() -> Self {
        Self {
            lines: Rope::new(),
            workspace: HashMap::new(),
        }
    }

    fn line_count(&self) -> usize {
        self.lines.len_lines().saturating_sub(1)
    }

    pub fn ml_append(&mut self, lnum: usize, line: &str) -> bool {
        if lnum > self.line_count() {
            return false;
        }
        let char_idx = self.lines.line_to_char(lnum);
        self.lines.insert(char_idx, &format!("{}\n", line));
        true
    }

    pub fn ml_delete(&mut self, lnum: usize) -> Option<String> {
        if lnum == 0 || lnum > self.line_count() {
            return None;
        }
        let start = self.lines.line_to_char(lnum - 1);
        let end = self.lines.line_to_char(lnum);
        let removed = self.lines.slice(start..end).to_string();
        self.lines.remove(start..end);
        Some(removed.trim_end_matches('\n').to_string())
    }

    pub fn ml_replace(&mut self, lnum: usize, line: &str) -> Option<String> {
        if lnum == 0 || lnum > self.line_count() {
            return None;
        }
        let start = self.lines.line_to_char(lnum - 1);
        let end = self.lines.line_to_char(lnum);
        let old = self.lines.slice(start..end).to_string();
        self.lines.remove(start..end);
        self.lines.insert(start, &format!("{}\n", line));
        Some(old.trim_end_matches('\n').to_string())
    }
}

fn rope_slice_to_cstring(slice: RopeSlice) -> CString {
    let mut s = slice.to_string();
    if s.ends_with('\n') {
        s.pop();
    }
    CString::new(s).unwrap()
}

#[no_mangle]
pub extern "C" fn ml_buffer_new() -> *mut MemBuffer {
    Box::into_raw(Box::new(MemBuffer::new()))
}

#[no_mangle]
pub extern "C" fn ml_buffer_free(ptr: *mut MemBuffer) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

#[no_mangle]
pub extern "C" fn ml_append(buf: *mut MemBuffer, lnum: usize, line: *const c_char) -> bool {
    if buf.is_null() || line.is_null() {
        return false;
    }
    let buffer = unsafe { &mut *buf };
    let c_str = unsafe { CStr::from_ptr(line) };
    match c_str.to_str() {
        Ok(s) if s.len() <= MAX_LINE_LEN => buffer.ml_append(lnum, s),
        _ => false,
    }
}

#[no_mangle]
pub extern "C" fn ml_delete(buf: *mut MemBuffer, lnum: usize) -> bool {
    if buf.is_null() {
        return false;
    }
    let buffer = unsafe { &mut *buf };
    buffer.ml_delete(lnum).is_some()
}

#[no_mangle]
pub extern "C" fn ml_replace(buf: *mut MemBuffer, lnum: usize, line: *const c_char) -> bool {
    if buf.is_null() || line.is_null() {
        return false;
    }
    let buffer = unsafe { &mut *buf };
    let c_str = unsafe { CStr::from_ptr(line) };
    match c_str.to_str() {
        Ok(s) if s.len() <= MAX_LINE_LEN => {
            buffer.ml_replace(lnum, s);
            true
        }
        _ => false,
    }
}

/// Get a pointer to a NUL-terminated, writable line buffer for lnum.
/// If `for_change` is true, the buffer may be modified by the caller.
/// The returned pointer remains valid until the next call that invalidates
/// the workspace for the same lnum (e.g. another ml_get_line or replace).
#[no_mangle]
pub extern "C" fn ml_get_line(
    buf: *mut MemBuffer,
    lnum: usize,
    for_change: bool,
    out_len: *mut usize,
) -> *mut u8 {
    if buf.is_null() {
        return b"\0".as_ptr() as *mut u8;
    }
    let b = unsafe { &mut *buf };
    if lnum == 0 || lnum > b.line_count() {
        if !out_len.is_null() {
            unsafe { *out_len = 0 };
        }
        return b"\0".as_ptr() as *mut u8;
    }
    let slice = b.lines.line(lnum - 1);
    let needed = rope_slice_to_cstring(slice).into_bytes_with_nul();
    let entry = b.workspace.entry(lnum).or_insert_with(|| needed.clone());
    if !for_change {
        if *entry != needed {
            *entry = needed;
        }
    } else if entry.len() < 2 {
        // Ensure there is at least space for one character plus NUL so that
        // callers writing the first byte do not drop the terminator.
        entry.clear();
        entry.extend_from_slice(&[0u8, 0u8]);
    }
    if !out_len.is_null() {
        unsafe { *out_len = entry.len().saturating_sub(1) };
    }
    entry.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn ml_line_count(buf: *const MemBuffer) -> usize {
    if buf.is_null() {
        return 0;
    }
    let b = unsafe { &*buf };
    b.line_count()
}

/// Representation of the initial swap file block.
#[repr(C)]
pub struct Block0 {
    pub version: u32,
    pub mtime: u64,
}

/// Representation of a pointer block in a swap file.
#[repr(C)]
pub struct PtrBlock {
    pub header: u32,
    pub pointers: [u32; 1],
}

/// Representation of a data block in a swap file.
#[repr(C)]
pub struct DataBlock {
    pub header: u32,
    pub data: [u8; 4096],
}

use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::io::Result;

/// Map a file into memory, creating it if needed.
pub fn map_file(path: &str, size: usize) -> Result<MmapMut> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;
    file.set_len(size as u64)?;
    unsafe { MmapMut::map_mut(&file) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn basic_editing() {
        let mut buf = MemBuffer::new();
        assert!(buf.ml_append(0, "hello"));
        assert!(buf.ml_append(1, "world"));
        assert_eq!(buf.ml_replace(2, "vim"), Some("world".into()));
        assert_eq!(buf.ml_delete(1), Some("hello".into()));
        assert_eq!(ml_line_count(&buf as *const _), 1);
        let mut len: usize = 0;
        let ptr = ml_get_line(&mut buf as *mut _, 1, false, &mut len as *mut usize);
        let s = unsafe { CStr::from_ptr(ptr as *const c_char) }
            .to_str()
            .unwrap();
        assert_eq!(s, "vim");
        assert_eq!(len, 3);
    }

    #[test]
    fn line_too_long() {
        let mut buf = MemBuffer::new();
        let long = vec![b'a'; MAX_LINE_LEN + 1];
        let c = std::ffi::CString::new(long).unwrap();
        assert!(!ml_append(&mut buf as *mut _, 0, c.as_ptr()));
    }

    #[test]
    fn count_lines() {
        let mut buf = MemBuffer::new();
        assert_eq!(ml_line_count(&buf as *const _), 0);
        assert!(ml_append(
            &mut buf as *mut _,
            0,
            b"one\0".as_ptr() as *const c_char
        ));
        assert_eq!(ml_line_count(&buf as *const _), 1);
    }
}
