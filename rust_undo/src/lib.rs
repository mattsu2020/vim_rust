use std::ffi::{CStr, CString};
use std::os::raw::{c_char};
use std::rc::Rc;
use std::cell::RefCell;

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

// Persistent node representing a change.
#[derive(Clone)]
struct Node {
    text: String,
    prev: Option<Rc<Node>>,
}

impl Node {
    fn new(text: String, prev: Option<Rc<Node>>) -> Rc<Node> {
        Rc::new(Node { text, prev })
    }
}

pub struct UndoHistory {
    current: Option<Rc<Node>>,
}

impl UndoHistory {
    fn new() -> Self {
        Self { current: None }
    }

    fn push(&mut self, text: &str) {
        let prev = self.current.clone();
        self.current = Some(Node::new(text.to_string(), prev));
    }

    fn pop(&mut self) -> Option<String> {
        if let Some(node) = self.current.clone() {
            self.current = node.prev.clone();
            Some(node.text.clone())
        } else {
            None
        }
    }
}

#[no_mangle]
pub extern "C" fn undo_history_new() -> *mut UndoHistory {
    Box::into_raw(Box::new(UndoHistory::new()))
}

#[no_mangle]
pub extern "C" fn undo_history_free(ptr: *mut UndoHistory) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)); }
    }
}

#[no_mangle]
pub extern "C" fn undo_push(ptr: *mut UndoHistory, text: *const c_char) -> bool {
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
pub extern "C" fn undo_pop(ptr: *mut UndoHistory, buf: *mut c_char, len: usize) -> bool {
    if ptr.is_null() || buf.is_null() {
        return false;
    }
    let hist = unsafe { &mut *ptr };
    if let Some(text) = hist.pop() {
        let s = match CString::new(text) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let bytes = s.as_bytes_with_nul();
        if bytes.len() > len {
            return false;
        }
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf as *mut u8, bytes.len()); }
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
        let hist = undo_history_new();
        let c1 = CString::new("one").unwrap();
        let c2 = CString::new("two").unwrap();
        assert!(undo_push(hist, c1.as_ptr()));
        assert!(undo_push(hist, c2.as_ptr()));
        let mut buf = [0i8; 10];
        assert!(undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        let s = unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap();
        assert_eq!(s, "two");
        assert!(undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        let s = unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap();
        assert_eq!(s, "one");
        assert!(!undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        undo_history_free(hist);
    }

    #[test]
    fn integrates_with_memline() {
        let mut buf = MemBuffer::new();
        let hist = undo_history_new();

        // Start with one line
        assert!(buf.ml_append(0, "hello"));

        // Replace line and record previous text for undo
        let previous = buf.ml_replace(1, "world").unwrap();
        let prev_c = CString::new(previous.clone()).unwrap();
        assert!(undo_push(hist, prev_c.as_ptr()));

        // Undo the change
        let mut out = [0i8; 16];
        assert!(undo_pop(hist, out.as_mut_ptr(), out.len()));
        let last = unsafe { CStr::from_ptr(out.as_ptr()) }.to_str().unwrap();
        assert_eq!(last, "hello");
        assert_eq!(buf.ml_replace(1, last), Some(String::from("world")));

        undo_history_free(hist);
    }
}
