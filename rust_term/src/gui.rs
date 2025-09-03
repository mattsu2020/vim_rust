use std::os::raw::{c_char, c_int};
use std::ptr;

pub struct Terminal;

#[no_mangle]
pub extern "C" fn rust_term_new() -> *mut Terminal {
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_free(_term: *mut Terminal) {}

#[no_mangle]
pub unsafe extern "C" fn rust_term_out_char(_term: *mut Terminal, _c: c_int) -> c_int {
    -1
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_out_flush(_term: *mut Terminal) -> c_int {
    -1
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_move_cursor(
    _term: *mut Terminal,
    _x: c_int,
    _y: c_int,
) -> c_int {
    -1
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_clear_screen(_term: *mut Terminal) -> c_int {
    -1
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_print(
    _term: *mut Terminal,
    _s: *const c_char,
) -> c_int {
    -1
}

#[no_mangle]
pub extern "C" fn rust_term_color_count() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_get_winsize(
    _width: *mut c_int,
    _height: *mut c_int,
) -> c_int {
    -1
}
