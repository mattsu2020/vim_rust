use libc::{c_char, c_int};
use std::ptr;

pub struct ScreenBuffer;

#[no_mangle]
pub extern "C" fn rs_screen_new(_width: c_int, _height: c_int) -> *mut ScreenBuffer {
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn rs_screen_free(_buf: *mut ScreenBuffer) {}

#[no_mangle]
pub extern "C" fn rs_screen_draw_text(
    _buf: *mut ScreenBuffer,
    _row: c_int,
    _col: c_int,
    _text: *const c_char,
    _attr: u8,
) {
}

#[no_mangle]
pub extern "C" fn rs_screen_clear_line(_buf: *mut ScreenBuffer, _row: c_int, _attr: u8) {}

#[no_mangle]
pub extern "C" fn rs_screen_highlight(
    _buf: *mut ScreenBuffer,
    _row: c_int,
    _col: c_int,
    _len: c_int,
    _attr: u8,
) {
}

pub type FlushCallback = extern "C" fn(row: c_int, text: *const c_char, attr: *const u8, len: c_int);

#[no_mangle]
pub extern "C" fn rs_screen_flush(_buf: *mut ScreenBuffer, _cb: Option<FlushCallback>) {}
