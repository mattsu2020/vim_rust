use libc::{c_char, c_int};
use rust_ui::{with_ui_mut, ScreenBuffer};
use std::ffi::CStr;

#[no_mangle]
pub extern "C" fn rs_draw_line(
    _buf: *mut ScreenBuffer,
    row: c_int,
    text: *const c_char,
    attr: u8,
) {
    if text.is_null() {
        return;
    }
    let c_str = unsafe { CStr::from_ptr(text) };
    if let Ok(s) = c_str.to_str() {
        let _ = with_ui_mut(|ui| {
            ui.clear_line(row as usize, attr);
            ui.draw_text(row as usize, 0, s, attr);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn draw_line_writes_text() {
        rust_ui::init(20, 2);
        let txt = CString::new("hello").unwrap();
        rs_draw_line(std::ptr::null_mut(), 1, txt.as_ptr(), 2);
        rust_ui::with_ui_mut(|ui| {
            assert_eq!(ui.line(1), "hello               ");
        })
        .unwrap();
    }
}
