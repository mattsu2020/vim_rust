use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::rc::Rc;

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
    seq: i64,
    prev: Option<Rc<Node>>,
}

impl Node {
    fn new(text: String, prev: Option<Rc<Node>>, seq: i64) -> Rc<Node> {
        Rc::new(Node { text, seq, prev })
    }
}

pub struct UndoHistory {
    current: Option<Rc<Node>>,
    next_seq: i64,
}

impl UndoHistory {
    pub(crate) fn new() -> Self {
        Self {
            current: None,
            next_seq: 1,
        }
    }

    pub(crate) fn push(&mut self, text: &str) {
        let prev = self.current.clone();
        let node = Node::new(text.to_string(), prev, self.next_seq);
        self.current = Some(node);
        self.next_seq += 1;
    }

    pub(crate) fn pop(&mut self) -> Option<String> {
        if let Some(node) = self.current.clone() {
            self.current = node.prev.clone();
            self.next_seq = node.seq;
            Some(node.text.clone())
        } else {
            None
        }
    }

    #[cfg(test)]
    pub(crate) fn verify_integrity(&self) -> bool {
        let mut expected = self.next_seq - 1;
        let mut cur = self.current.clone();
        while let Some(node) = cur {
            if node.seq != expected {
                return false;
            }
            expected -= 1;
            cur = node.prev.clone();
        }
        true
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
    if let Some(text) = hist.pop() {
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
        unsafe {
            assert!((*hist).verify_integrity());
        }
        let c1 = CString::new("one").unwrap();
        let c2 = CString::new("two").unwrap();
        assert!(rs_undo_push(hist, c1.as_ptr()));
        unsafe {
            assert!((*hist).verify_integrity());
        }
        assert!(rs_undo_push(hist, c2.as_ptr()));
        unsafe {
            assert!((*hist).verify_integrity());
        }
        let mut buf = [0i8; 10];
        assert!(rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        unsafe {
            assert!((*hist).verify_integrity());
            let s = CStr::from_ptr(buf.as_ptr()).to_str().unwrap();
            assert_eq!(s, "two");
        }
        assert!(rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        unsafe {
            assert!((*hist).verify_integrity());
            let s = CStr::from_ptr(buf.as_ptr()).to_str().unwrap();
            assert_eq!(s, "one");
        }
        assert!(!rs_undo_pop(hist, buf.as_mut_ptr(), buf.len()));
        unsafe {
            assert!((*hist).verify_integrity());
        }
        rs_undo_history_free(hist);
    }

    #[test]
    fn history_integrity_check() {
        let mut hist = UndoHistory::new();
        assert!(hist.verify_integrity());
        hist.push("alpha");
        hist.push("beta");
        assert!(hist.verify_integrity());
        assert_eq!(hist.pop(), Some("beta".to_string()));
        assert!(hist.verify_integrity());
        hist.push("gamma");
        assert!(hist.verify_integrity());
        assert_eq!(hist.pop(), Some("gamma".to_string()));
        assert_eq!(hist.pop(), Some("alpha".to_string()));
        assert!(hist.verify_integrity());
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
}
