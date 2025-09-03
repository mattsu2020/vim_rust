use libc::{c_char, c_int};
use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::{LazyLock, Mutex};

/// Logging level used for queued messages.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info = 0,
    Warn = 1,
    Error = 2,
}

struct QueuedMsg {
    text: CString,
    level: LogLevel,
}

// Global queue storing pending messages.
static MSG_QUEUE: LazyLock<Mutex<VecDeque<QueuedMsg>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

// Last error message recorded, if any.
static LAST_ERROR: LazyLock<Mutex<Option<CString>>> =
    LazyLock::new(|| Mutex::new(None));

// Very small translation table for demonstration purposes.
fn translate(msg: &str) -> String {
    let lang = std::env::var("LANG").unwrap_or_default();
    if lang.starts_with("ja") {
        match msg {
            "Hello" => "こんにちは".to_string(),
            "Error" => "エラー".to_string(),
            _ => msg.to_string(),
        }
    } else {
        msg.to_string()
    }
}

/// Enqueue a message with the given log level.
#[no_mangle]
pub unsafe extern "C" fn rs_queue_message(msg: *const c_char, level: c_int) {
    if msg.is_null() {
        return;
    }
    let cstr = CStr::from_ptr(msg);
    let text = translate(cstr.to_str().unwrap_or(""));
    let cstring = match CString::new(text) {
        Ok(cs) => cs,
        Err(_) => return,
        };
    let lvl = match level {
        1 => LogLevel::Warn,
        2 => LogLevel::Error,
        _ => LogLevel::Info,
    };

    if lvl == LogLevel::Error {
        *LAST_ERROR.lock().unwrap() = Some(cstring.clone());
    }
    MSG_QUEUE
        .lock()
        .unwrap()
        .push_back(QueuedMsg { text: cstring, level: lvl });
}

/// Write raw text coming from the C UI layer.  The text is enqueued as an
/// info-level message so that Rust-side code can display or process it.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_write(msg: *const c_char, len: c_int) {
    if msg.is_null() || len <= 0 {
        return;
    }
    let cstr = CStr::from_ptr(msg);
    let text = translate(cstr.to_str().unwrap_or(""));
    let cstring = match CString::new(text) {
        Ok(cs) => cs,
        Err(_) => return,
    };
    MSG_QUEUE
        .lock()
        .unwrap()
        .push_back(QueuedMsg { text: cstring, level: LogLevel::Info });
}

/// Pop the next queued message.  Returns a newly allocated C string that must be
/// freed with `rs_free_cstring`.  When `level` is not NULL the log level is
/// written there.  Returns NULL when the queue is empty.
#[no_mangle]
pub unsafe extern "C" fn rs_pop_message(level: *mut c_int) -> *mut c_char {
    let mut queue = MSG_QUEUE.lock().unwrap();
    if let Some(msg) = queue.pop_front() {
        if !level.is_null() {
            *level = msg.level as c_int;
        }
        return CString::into_raw(msg.text);
    }
    ptr::null_mut()
}

/// Return a pointer to the last error message or NULL when none was recorded.
#[no_mangle]
pub unsafe extern "C" fn rs_get_last_error() -> *const c_char {
    let guard = LAST_ERROR.lock().unwrap();
    match guard.as_ref() {
        Some(cs) => cs.as_ptr(),
        None => ptr::null(),
    }
}

/// Clear all queued messages and the last error.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_messages() {
    MSG_QUEUE.lock().unwrap().clear();
    LAST_ERROR.lock().unwrap().take();
}

/// Free a C string that originated from this module.
#[no_mangle]
pub unsafe extern "C" fn rs_free_cstring(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    let _ = CString::from_raw(s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_level_and_i18n() {
        unsafe {
            rs_clear_messages();
            std::env::set_var("LANG", "ja_JP.UTF-8");
            let msg = CString::new("Hello").unwrap();
            rs_queue_message(msg.as_ptr(), LogLevel::Info as c_int);
            let mut lvl = -1;
            let ptr = rs_pop_message(&mut lvl as *mut c_int);
            let rust_str = CStr::from_ptr(ptr).to_str().unwrap().to_string();
            rs_free_cstring(ptr);
            assert_eq!(rust_str, "こんにちは");
            assert_eq!(lvl, LogLevel::Info as c_int);

            rs_clear_messages();
            std::env::set_var("LANG", "C");
            let err = CString::new("failure").unwrap();
            rs_queue_message(err.as_ptr(), LogLevel::Error as c_int);
            let mut lvl2 = -1;
            let ptr2 = rs_pop_message(&mut lvl2 as *mut c_int);
            let last_err = CStr::from_ptr(rs_get_last_error()).to_str().unwrap();
            rs_free_cstring(ptr2);
            assert_eq!(lvl2, LogLevel::Error as c_int);
            assert_eq!(last_err, "failure");
        }
    }

    #[test]
    fn ui_write_enqueues() {
        unsafe {
            rs_clear_messages();
            let msg = CString::new("hi").unwrap();
            rs_ui_write(msg.as_ptr(), 2);
            let mut lvl = -1;
            let ptr = rs_pop_message(&mut lvl as *mut c_int);
            assert!(!ptr.is_null());
            rs_free_cstring(ptr);
            assert_eq!(lvl, LogLevel::Info as c_int);
        }
    }
}

