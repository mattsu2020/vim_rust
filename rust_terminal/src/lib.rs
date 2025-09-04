pub use rust_term::*;
pub use rust_termlib::*;

use std::ffi::CString;
use std::os::raw::{c_int, c_void};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_roundtrip() {
        let term = rust_terminal_new();
        unsafe {
            assert!(rust_terminal_get_status_text(term).is_null());
            rust_terminal_clear_status_text(term);
            rust_terminal_free(term);
        }
    }
}
