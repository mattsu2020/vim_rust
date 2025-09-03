use std::collections::{BTreeMap, HashMap};
use std::ffi::{CStr};
use std::os::raw::c_char;

/// Simple in-memory representation of lines in a buffer.
#[derive(Default)]
pub struct MemBuffer {
    lines: BTreeMap<usize, String>,
    // Workspace for exposing lines to C as modifiable C strings.
    // Keyed by lnum. Stored with a trailing NUL.
    workspace: HashMap<usize, Vec<u8>>,
}

impl MemBuffer {
    pub fn new() -> Self {
        Self { lines: BTreeMap::new(), workspace: HashMap::new() }
    }

    pub fn ml_append(&mut self, lnum: usize, line: &str) -> bool {
        // Valid line numbers are in the range [0, line_count].  Appending past
        // the end of the buffer is an error.
        if lnum > self.lines.len() {
            return false;
        }

        let insert_at = lnum + 1;
        let mut tail: BTreeMap<usize, String> = self.lines.split_off(&insert_at);
        self.lines.insert(insert_at, line.to_string());
        for (i, (_, l)) in tail.into_iter().enumerate() {
            self.lines.insert(insert_at + 1 + i, l);
        }
        true
    }

    pub fn ml_delete(&mut self, lnum: usize) -> Option<String> {
        if lnum == 0 || lnum > self.lines.len() {
            return None;
        }

        let removed = self.lines.remove(&lnum);
        if removed.is_some() {
            let keys: Vec<usize> =
                self.lines.range(lnum + 1..).map(|(&k, _)| k).collect();
            for k in keys {
                if let Some(v) = self.lines.remove(&k) {
                    self.lines.insert(k - 1, v);
                }
            }
        }
        removed
    }

    pub fn ml_replace(&mut self, lnum: usize, line: &str) -> Option<String> {
        if lnum == 0 || lnum > self.lines.len() {
            return None;
        }
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

/// Get a pointer to a NUL-terminated, writable line buffer for lnum.
/// If `for_change` is true, the buffer may be modified by the caller.
/// The returned pointer remains valid until the next call that invalidates
/// the workspace for the same lnum (e.g. another rs_ml_get_line or replace).
#[no_mangle]
pub extern "C" fn rs_ml_get_line(
    buf: *mut MemBuffer,
    lnum: usize,
    for_change: bool,
    out_len: *mut usize,
) -> *mut u8 {
    if buf.is_null() {
        return b"\0".as_ptr() as *mut u8;
    }
    let b = unsafe { &mut *buf };
    let content = b.lines.get(&lnum).map(|s| s.as_str()).unwrap_or("");
    let entry = b.workspace.entry(lnum).or_insert_with(|| {
        let mut v = Vec::with_capacity(content.len() + 1);
        v.extend_from_slice(content.as_bytes());
        v.push(0);
        v
    });
    if !for_change {
        // Ensure workspace matches current content when not changing.
        // If size differs, refresh.
        let needed = content.as_bytes();
        if entry.len() != needed.len() + 1 || &entry[..needed.len()] != needed {
            entry.clear();
            entry.extend_from_slice(needed);
            entry.push(0);
        }
    } else {
        // Ensure there is at least space for one character plus NUL so that
        // callers writing the first byte do not drop the terminator.
        if entry.len() < 2 {
            entry.clear();
            entry.extend_from_slice(&[0u8, 0u8]);
        }
    }
    if !out_len.is_null() {
        unsafe { *out_len = entry.len().saturating_sub(1) };
    }
    entry.as_mut_ptr()
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
