use std::ffi::CString;
use std::os::raw::{c_int, c_void};
use std::ptr;

use vte::{Params, Parser};

/// Internal terminal state backed by the `vte` parser.
pub struct TermState {
    parser: Parser,
    screen: Vec<u8>,
    screen_cstr: Option<CString>,
}

impl TermState {
    fn new() -> Self {
        Self {
            parser: Parser::new(),
            screen: Vec::new(),
            screen_cstr: None,
        }
    }

    fn feed(&mut self, bytes: &[u8]) {
        let mut parser = std::mem::take(&mut self.parser);
        parser.advance(self, bytes);
        self.parser = parser;
    }

    fn screen_ptr(&mut self) -> *const u8 {
        if self.screen_cstr.is_none() {
            self.screen_cstr = CString::new(self.screen.clone()).ok();
        }
        self.screen_cstr
            .as_ref()
            .map(|c| c.as_ptr() as *const u8)
            .unwrap_or(ptr::null())
    }
}

impl vte::Perform for TermState {
    fn print(&mut self, c: char) {
        self.screen
            .extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
        self.screen_cstr = None;
    }

    fn execute(&mut self, byte: u8) {
        if byte == b'\n' {
            self.screen.push(b'\n');
            self.screen_cstr = None;
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, byte: u8) {
        self.screen.push(byte);
        self.screen_cstr = None;
    }
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
    fn csi_dispatch(
        &mut self,
        _params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        _action: char,
    ) {
    }
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _action: u8) {}
}

#[no_mangle]
pub extern "C" fn rust_terminal_new() -> *mut TermState {
    Box::into_raw(Box::new(TermState::new()))
}

#[no_mangle]
/// # Safety
///
/// `term` must be a valid pointer returned by [`rust_terminal_new`]. After
/// calling this function the pointer must not be used again.
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
/// # Safety
///
/// The pointer is not dereferenced and may be null or dangling.
pub unsafe extern "C" fn rust_terminal_none_open(_term: *mut c_void) -> c_int {
    -1
}

/// Feed input bytes to the terminal parser.
#[no_mangle]
/// # Safety
///
/// `term` must be valid and `data` must point to `len` bytes.
pub unsafe extern "C" fn rust_terminal_feed(term: *mut TermState, data: *const u8, len: usize) {
    if term.is_null() || data.is_null() {
        return;
    }
    let ts = &mut *term;
    let slice = std::slice::from_raw_parts(data, len);
    ts.feed(slice);
}

/// Return the parsed screen contents as a C string.
#[no_mangle]
/// # Safety
///
/// `term` must be valid and the returned pointer is only valid until the next
/// mutating call on `term`.
pub unsafe extern "C" fn rust_terminal_get_screen_text(term: *mut TermState) -> *const u8 {
    if term.is_null() {
        return ptr::null();
    }
    let ts = &mut *term;
    ts.screen_ptr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn feeds_and_collects_output() {
        let term = rust_terminal_new();
        unsafe {
            let input = b"hi\n";
            rust_terminal_feed(term, input.as_ptr(), input.len());
            let ptr = rust_terminal_get_screen_text(term);
            let s = CStr::from_ptr(ptr as *const i8).to_str().unwrap();
            assert_eq!(s, "hi\n");
            rust_terminal_free(term);
        }
    }
}
