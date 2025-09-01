use libc::{c_char, c_int, size_t};
use std::ffi::{CStr, CString};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// Callback type invoked when a line is produced by the terminal.
pub type LineCallback = extern "C" fn(*const c_char);

/// Terminal structure managed from Rust.  It collects scrollback and
/// optionally forwards each produced line to a callback registered from C.
pub struct Terminal {
    tx: Option<mpsc::Sender<String>>,
    scrollback: Arc<Mutex<Vec<String>>>,
    callback: Arc<Mutex<Option<LineCallback>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Terminal {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel::<String>();
        let scrollback = Arc::new(Mutex::new(Vec::new()));
        let sb_clone = scrollback.clone();
        let callback: Arc<Mutex<Option<LineCallback>>> = Arc::new(Mutex::new(None));
        let cb_clone = callback.clone();
        let handle = thread::spawn(move || {
            while let Ok(line) = rx.recv() {
                sb_clone.lock().unwrap().push(line.clone());
                if let Some(cb) = *cb_clone.lock().unwrap() {
                    if let Ok(c_string) = CString::new(line) {
                        cb(c_string.as_ptr());
                    }
                }
            }
        });
        Self { tx: Some(tx), scrollback, callback, handle: Some(handle) }
    }

    fn write(&self, text: &str) {
        if let Some(ref tx) = self.tx {
            let _ = tx.send(text.to_string());
        }
    }

    fn set_callback(&self, cb: LineCallback) {
        *self.callback.lock().unwrap() = Some(cb);
    }

    fn scrollback_line(&self, idx: usize) -> Option<String> {
        self.scrollback.lock().unwrap().get(idx).cloned()
    }
}

#[no_mangle]
pub extern "C" fn terminal_full_new(_cols: c_int, _rows: c_int) -> *mut Terminal {
    Box::into_raw(Box::new(Terminal::new()))
}

#[no_mangle]
pub extern "C" fn terminal_full_free(term: *mut Terminal) {
    if term.is_null() { return; }
    unsafe {
        let mut boxed = Box::from_raw(term);
        // Dropping the sender closes the channel and terminates the thread loop.
        boxed.tx.take();
        if let Some(handle) = boxed.handle.take() {
            let _ = handle.join();
        }
        // boxed drops here
    }
}

#[no_mangle]
pub extern "C" fn terminal_full_write(term: *mut Terminal, data: *const c_char) {
    if term.is_null() { return; }
    let c_str = unsafe { CStr::from_ptr(data) };
    if let Ok(text) = c_str.to_str() {
        unsafe { &*term }.write(text);
    }
}

#[no_mangle]
pub extern "C" fn terminal_full_set_callback(term: *mut Terminal, cb: LineCallback) {
    if term.is_null() { return; }
    unsafe { &*term }.set_callback(cb);
}

#[no_mangle]
pub extern "C" fn terminal_full_get_scrollback(
    term: *mut Terminal,
    idx: size_t,
    buf: *mut c_char,
    buf_len: size_t,
) -> size_t {
    if term.is_null() || buf.is_null() { return 0; }
    let line = unsafe { &*term }.scrollback_line(idx as usize);
    if let Some(line) = line {
        if let Ok(c_string) = CString::new(line) {
            let bytes = c_string.as_bytes_with_nul();
            let copy_len = std::cmp::min(bytes.len(), buf_len as usize);
            unsafe {
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    buf as *mut u8,
                    copy_len,
                );
            }
            return copy_len as size_t;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    extern "C" fn capture(_: *const c_char) {}

    #[test]
    fn records_lines_and_callback() {
        let term = unsafe { terminal_full_new(80, 24) };
        unsafe { terminal_full_set_callback(term, capture); }
        let msg = CString::new("hello").unwrap();
        unsafe { terminal_full_write(term, msg.as_ptr()); }
        thread::sleep(std::time::Duration::from_millis(10));
        let mut buf = [0i8; 16];
        let copied = unsafe { terminal_full_get_scrollback(term, 0, buf.as_mut_ptr(), buf.len()) };
        assert!(copied > 0);
        let content = unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap();
        assert_eq!(content, "hello");
        unsafe { terminal_full_free(term) };
    }
}
