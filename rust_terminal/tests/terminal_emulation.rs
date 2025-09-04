use std::ffi::CStr;

use rust_terminal::{
    rust_terminal_feed, rust_terminal_free, rust_terminal_get_screen_text, rust_terminal_new,
};

#[test]
fn parses_plain_text() {
    let term = rust_terminal_new();
    unsafe {
        let input = b"hello\n";
        rust_terminal_feed(term, input.as_ptr(), input.len());
        let ptr = rust_terminal_get_screen_text(term);
        let s = CStr::from_ptr(ptr as *const i8).to_str().unwrap();
        assert_eq!(s, "hello\n");
        rust_terminal_free(term);
    }
}

#[test]
fn ignores_ansi_escape_sequences() {
    let term = rust_terminal_new();
    unsafe {
        let input = b"\x1b[31mred\x1b[0m";
        rust_terminal_feed(term, input.as_ptr(), input.len());
        let ptr = rust_terminal_get_screen_text(term);
        let s = CStr::from_ptr(ptr as *const i8).to_str().unwrap();
        assert_eq!(s, "red");
        rust_terminal_free(term);
    }
}
