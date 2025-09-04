use libc::{c_char, c_int};
use rust_screen::ScreenBuffer;

#[no_mangle]
pub extern "C" fn rs_draw_line(
    buf: *mut ScreenBuffer,
    row: c_int,
    text: *const c_char,
    attr: u8,
) {
    if buf.is_null() || text.is_null() {
        return;
    }
    // Clear the target line and draw new text starting at column 0.
    rust_screen::rs_screen_clear_line(buf, row, attr);
    rust_screen::rs_screen_draw_text(buf, row, 0, text, attr);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn draw_line_writes_text() {
        let mut sb = ScreenBuffer::new(20, 2);
        let txt = CString::new("hello").unwrap();
        rs_draw_line(&mut sb as *mut ScreenBuffer, 1, txt.as_ptr(), 2);
        assert_eq!(sb.line_as_string(1), "hello               ");
    }
}
