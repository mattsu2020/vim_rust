use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[repr(C)]
pub struct UEntry {
    pub next: *mut UEntry,
    pub line: *mut c_char,
}

#[repr(C)]
pub struct UHeader {
    pub next: *mut UHeader,
    pub prev: *mut UHeader,
    pub entries: *mut UEntry,
    pub seq: i64,
}

// Simple undo/redo history implemented as two stacks.  The undo stack holds
// the most recently pushed changes.  When undoing, an entry is moved to the
// redo stack.  Pushing a new change clears the redo stack, as making a new
// change after undoing discards the redo history.
pub struct UndoHistory {
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
}

impl UndoHistory {
    fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn push(&mut self, text: &str) {
        self.undo_stack.push(text.to_string());
        self.redo_stack.clear();
    }

    fn undo(&mut self) -> Option<String> {
        self.undo_stack.pop().map(|s| {
            self.redo_stack.push(s.clone());
            s
        })
    }

    fn redo(&mut self) -> Option<String> {
        self.redo_stack.pop().map(|s| {
            self.undo_stack.push(s.clone());
            s
        })
    }
}

#[no_mangle]
pub extern "C" fn rs_undo_history_new() -> *mut UndoHistory {
    Box::into_raw(Box::new(UndoHistory::new()))
}

#[no_mangle]
pub extern "C" fn rs_undo_history_free(ptr: *mut UndoHistory) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_undo_push(ptr: *mut UndoHistory, text: *const c_char) -> bool {
    if ptr.is_null() || text.is_null() {
        return false;
    }
    let hist = unsafe { &mut *ptr };
    let c_str = unsafe { CStr::from_ptr(text) };
    if let Ok(s) = c_str.to_str() {
        hist.push(s);
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn rs_undo_pop(ptr: *mut UndoHistory, buf: *mut c_char, len: usize) -> bool {
    if ptr.is_null() || buf.is_null() {
        return false;
    }
    let hist = unsafe { &mut *ptr };
    if let Some(text) = hist.undo() {
        let s = match CString::new(text) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let bytes = s.as_bytes_with_nul();
        if bytes.len() > len {
            return false;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf as *mut u8, bytes.len());
        }
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn rs_undo_redo(ptr: *mut UndoHistory, buf: *mut c_char, len: usize) -> bool {
    if ptr.is_null() || buf.is_null() {
        return false;
    }
    let hist = unsafe { &mut *ptr };
    if let Some(text) = hist.redo() {
        let s = match CString::new(text) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let bytes = s.as_bytes_with_nul();
        if bytes.len() > len {
            return false;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf as *mut u8, bytes.len());
        }
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_memline::MemBuffer;

    #[test]
    fn push_and_pop_changes() {
        let hist = rs_undo_history_new();
        let c1 = CString::new("one").unwrap();
        let c2 = CString::new("two").unwrap();
        assert!(rs_undo_push(hist, c1.as_ptr()));
        assert!(rs_undo_push(hist, c2.as_ptr()));
        let mut buf = [0i8; 10];
        assert!(rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        let s = unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap();
        assert_eq!(s, "two");
        assert!(rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        let s = unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap();
        assert_eq!(s, "one");
        assert!(!rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        rs_undo_history_free(hist);
    }

    #[test]
    fn integrates_with_memline() {
        let mut buf = MemBuffer::new();
        let hist = rs_undo_history_new();

        // Start with one line
        assert!(buf.ml_append(0, "hello"));

        // Replace line and record previous text for undo
        let previous = buf.ml_replace(1, "world").unwrap();
        let prev_c = CString::new(previous.clone()).unwrap();
        assert!(rs_undo_push(hist, prev_c.as_ptr()));

        // Undo the change
        let mut out = [0i8; 16];
        assert!(rs_undo_pop(hist, out.as_mut_ptr(), out.len()));
        let last = unsafe { CStr::from_ptr(out.as_ptr()) }.to_str().unwrap();
        assert_eq!(last, "hello");
        assert_eq!(buf.ml_replace(1, last), Some(String::from("world")));

        rs_undo_history_free(hist);
    }

    #[test]
    fn undo_redo_edge_cases() {
        let hist = rs_undo_history_new();
        let one = CString::new("one").unwrap();
        let two = CString::new("two").unwrap();
        let three = CString::new("three").unwrap();
        assert!(rs_undo_push(hist, one.as_ptr()));
        assert!(rs_undo_push(hist, two.as_ptr()));
        assert!(rs_undo_push(hist, three.as_ptr()));

        let mut buf = [0i8; 16];
        // Undo returns in reverse order.
        assert!(rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        assert_eq!(
            unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap(),
            "three"
        );
        assert!(rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        assert_eq!(
            unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap(),
            "two"
        );
        // Redo restores in original order.
        assert!(rs_undo_redo(hist, buf.as_mut_ptr(), buf.len()));
        assert_eq!(
            unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap(),
            "two"
        );
        assert!(rs_undo_redo(hist, buf.as_mut_ptr(), buf.len()));
        assert_eq!(
            unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap(),
            "three"
        );
        // Redo beyond history fails.
        assert!(!rs_undo_redo(hist, buf.as_mut_ptr(), buf.len()));

        // Undo all entries again and ensure further undo fails.
        assert!(rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        assert_eq!(
            unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap(),
            "three"
        );
        assert!(rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        assert_eq!(
            unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap(),
            "two"
        );
        assert!(rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        assert_eq!(
            unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap(),
            "one"
        );
        assert!(!rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));

        rs_undo_history_free(hist);
    }

    #[test]
    fn push_clears_redo_stack() {
        let hist = rs_undo_history_new();
        let a = CString::new("a").unwrap();
        let b = CString::new("b").unwrap();
        let c = CString::new("c").unwrap();
        assert!(rs_undo_push(hist, a.as_ptr()));
        assert!(rs_undo_push(hist, b.as_ptr()));
        let mut buf = [0i8; 16];
        assert!(rs_undo_pop(hist, buf.as_mut_ptr(), buf.len())); // undo b
                                                                 // Pushing a new entry should clear redo stack.
        assert!(rs_undo_push(hist, c.as_ptr()));
        assert!(!rs_undo_redo(hist, buf.as_mut_ptr(), buf.len()));

        // Undo and redo behaviour after clearing.
        assert!(rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        assert_eq!(
            unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap(),
            "c"
        );
        assert!(rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        assert_eq!(
            unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap(),
            "a"
        );
        assert!(!rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));

        rs_undo_history_free(hist);
    }

    #[test]
    fn empty_history_behaviour() {
        let hist = rs_undo_history_new();
        let mut buf = [0i8; 8];
        assert!(!rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        assert!(!rs_undo_redo(hist, buf.as_mut_ptr(), buf.len()));
        rs_undo_history_free(hist);
    }
}
