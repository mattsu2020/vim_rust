use libc::{c_char, c_int};
use rust_screen::ScreenBuffer;

/// Draws `text` on `screen` at the given `row`.
/// This is a very small subset of the original drawline.c logic.
#[no_mangle]
pub extern "C" fn rs_draw_line(screen: *mut ScreenBuffer, row: c_int, text: *const c_char) {
    if screen.is_null() || text.is_null() {
        return;
    }
    rust_screen::rs_screen_draw_text(screen, row, 0, text, 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn draw_line_writes_text() {
        let mut sb = ScreenBuffer::new(20, 2);
        let c_text = CString::new("hello").unwrap();
        rs_draw_line(&mut sb as *mut ScreenBuffer, 1, c_text.as_ptr());
        assert_eq!(sb.line_as_string(1).trim_end(), "hello");
    }
}
