use std::collections::BTreeMap;
use std::ffi::{CStr};
use std::os::raw::c_char;

/// Simple in-memory representation of lines in a buffer.
#[derive(Default)]
pub struct MemBuffer {
    lines: BTreeMap<usize, String>,
}

impl MemBuffer {
    pub fn new() -> Self {
        Self { lines: BTreeMap::new() }
    }

    pub fn ml_append(&mut self, lnum: usize, line: &str) -> bool {
        let insert_at = lnum + 1;
        let mut tail: BTreeMap<usize, String> = self.lines.split_off(&insert_at);
        self.lines.insert(insert_at, line.to_string());
        for (i, (_, l)) in tail.into_iter().enumerate() {
            self.lines.insert(insert_at + 1 + i, l);
        }
        true
    }

    pub fn ml_delete(&mut self, lnum: usize) -> Option<String> {
        let removed = self.lines.remove(&lnum);
        if removed.is_some() {
            let mut keys: Vec<usize> = self.lines.range(lnum + 1..).map(|(&k, _)| k).collect();
            for k in keys {
                if let Some(v) = self.lines.remove(&k) {
                    self.lines.insert(k - 1, v);
                }
            }
        }
        removed
    }

    pub fn ml_replace(&mut self, lnum: usize, line: &str) -> Option<String> {
        self.lines.insert(lnum, line.to_string())
    }
}

#[no_mangle]
pub extern "C" fn rs_ml_buffer_new() -> *mut MemBuffer {
    Box::into_raw(Box::new(MemBuffer::new()))
}

#[no_mangle]
pub extern "C" fn rs_ml_buffer_free(ptr: *mut MemBuffer) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)); }
    }
}

#[no_mangle]
pub extern "C" fn rs_ml_append(buf: *mut MemBuffer, lnum: usize, line: *const c_char) -> bool {
    if buf.is_null() || line.is_null() {
        return false;
    }
    let buffer = unsafe { &mut *buf };
    let c_str = unsafe { CStr::from_ptr(line) };
    match c_str.to_str() {
        Ok(s) => buffer.ml_append(lnum, s),
        Err(_) => false,
    }
}

#[no_mangle]
pub extern "C" fn rs_ml_delete(buf: *mut MemBuffer, lnum: usize) -> bool {
    if buf.is_null() {
        return false;
    }
    let buffer = unsafe { &mut *buf };
    buffer.ml_delete(lnum).is_some()
}

#[no_mangle]
pub extern "C" fn rs_ml_replace(buf: *mut MemBuffer, lnum: usize, line: *const c_char) -> bool {
    if buf.is_null() || line.is_null() {
        return false;
    }
    let buffer = unsafe { &mut *buf };
    let c_str = unsafe { CStr::from_ptr(line) };
    match c_str.to_str() {
        Ok(s) => { buffer.ml_replace(lnum, s); true },
        Err(_) => false,
    }
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

use std::fs::OpenOptions;
use memmap2::MmapMut;
use std::io::{Result};

/// Map a file into memory, creating it if needed.
pub fn map_file(path: &str, size: usize) -> Result<MmapMut> {
    let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
    file.set_len(size as u64)?;
    unsafe { MmapMut::map_mut(&file) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_editing() {
        let mut buf = MemBuffer::new();
        assert!(buf.ml_append(0, "hello"));
        assert!(buf.ml_append(1, "world"));
        assert_eq!(buf.ml_replace(2, "vim"), Some("world".into()));
        assert_eq!(buf.ml_delete(1), Some("hello".into()));
    }
}
