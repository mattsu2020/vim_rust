pub use rust_term::*;
pub use rust_termlib::*;

use libc::{c_char, c_int, c_void, strlen};
use rust_ui::ui_write;
use std::ffi::{CStr, CString};
use std::ptr;

/// Internal terminal state.
pub struct TermState {
    status: Option<CString>,
}

impl TermState {
    fn new() -> Self {
        Self { status: None }
    }
}

#[no_mangle]
pub extern "C" fn rust_terminal_new() -> *mut TermState {
    Box::into_raw(Box::new(TermState::new()))
}

#[no_mangle]
pub unsafe extern "C" fn rust_terminal_free(term: *mut TermState) {
    if term.is_null() {
        return;
    }
    drop(Box::from_raw(term));
}

/// Free unused terminals - placeholder implementation.
#[no_mangle]
pub extern "C" fn rust_terminal_free_unused() {}

/// Placeholder returning FAIL to indicate no terminal opened.
#[no_mangle]
pub unsafe extern "C" fn rust_terminal_none_open(_term: *mut c_void) -> c_int {
    -1
}

/// Clear cached status text for the terminal.
#[no_mangle]
pub unsafe extern "C" fn rust_terminal_clear_status_text(term: *mut TermState) {
    if let Some(ts) = term.as_mut() {
        ts.status = None;
    }
}

/// Return status text or NULL if none.
#[no_mangle]
pub unsafe extern "C" fn rust_terminal_get_status_text(term: *mut TermState) -> *const u8 {
    term
        .as_ref()
        .and_then(|ts| ts.status.as_ref().map(|s| s.as_ptr() as *const u8))
        .unwrap_or(ptr::null())
}

/// Set the status text for a terminal instance.
#[no_mangle]
pub unsafe extern "C" fn rust_terminal_set_status_text(term: *mut TermState, msg: *const c_char) {
    if term.is_null() || msg.is_null() {
        return;
    }
    if let Some(ts) = term.as_mut() {
        if let Ok(s) = CStr::from_ptr(msg).to_str() {
            ts.status = Some(CString::new(s).unwrap());
        }
    }
}

/// Write a message to the UI and remember it as status text.
#[no_mangle]
pub unsafe extern "C" fn rust_terminal_print(term: *mut TermState, msg: *const c_char) {
    if msg.is_null() {
        return;
    }
    let len = strlen(msg) as c_int;
    ui_write(msg, len);
    rust_terminal_set_status_text(term, msg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn status_roundtrip() {
        rust_ui::init(10, 2);
        let term = rust_terminal_new();
        unsafe {
            let msg = CString::new("hi").unwrap();
            rust_terminal_print(term, msg.as_ptr());
            let out = rust_ui::take_output();
            assert_eq!(out, vec!["hi".to_string()]);
            let ptr = rust_terminal_get_status_text(term) as *const c_char;
            assert_eq!(CStr::from_ptr(ptr).to_str().unwrap(), "hi");
            rust_terminal_clear_status_text(term);
            assert!(rust_terminal_get_status_text(term).is_null());
            rust_terminal_free(term);
        }
    }
}
