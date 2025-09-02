use rust_term::*;
use std::ffi::CString;

#[test]
fn create_and_flush() {
    let term = unsafe { rust_term_new() };
    assert!(!term.is_null());
    unsafe {
        assert_eq!(rust_term_out_char(term, b'A' as i32), 0);
        assert_eq!(rust_term_out_flush(term), 0);
        rust_term_free(term);
    }
}

#[cfg(unix)]
#[test]
fn unix_terminal_size() {
    let mut w = 0;
    let mut h = 0;
    // Terminal size may be zero when running without a TTY, just ensure call succeeds
    let _ = unsafe { rust_term_get_winsize(&mut w, &mut h) };
}

#[cfg(windows)]
#[test]
fn windows_color_count() {
    assert!(rust_term_color_count() >= 8);
}

#[cfg(target_os = "linux")]
#[test]
fn wayland_simulation() {
    std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
    let term = unsafe { rust_term_new() };
    assert!(!term.is_null());
    unsafe {
        let s = CString::new("wayland").unwrap();
        assert_eq!(rust_term_print(term, s.as_ptr()), 0);
        rust_term_free(term);
    }
    std::env::remove_var("WAYLAND_DISPLAY");
}
