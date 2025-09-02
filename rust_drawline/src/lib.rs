use libc::{c_char, c_int};
use std::ffi::CStr;
use rust_screen::ScreenBuffer;
use rust_syntax::{rs_syn_update, rs_syntax_start};

/// Draw a line of text into the given ScreenBuffer.
/// Each character is highlighted using a very simple rule: digits are drawn
/// with attribute `2` and all other characters with attribute `1`.
/// This demonstrates integrating syntax highlighting on the Rust side while
/// sharing the `ScreenBuffer` type with `screen.c` and `drawscreen.c` through
/// FFI.
#[no_mangle]
pub extern "C" fn rs_draw_line(
    buf: *mut ScreenBuffer,
    row: c_int,
    line: *const c_char,
) -> c_int {
    if buf.is_null() || line.is_null() {
        return row;
    }

    // Start syntax processing for this line (dummy window pointer and lnum).
    rs_syntax_start(std::ptr::null_mut(), row as i64);

    let screen = unsafe { &mut *buf };
    let c_line = unsafe { CStr::from_ptr(line) };
    if let Ok(text) = c_line.to_str() {
        let mut col = 0;
        for ch in text.chars() {
            let attr = if ch.is_ascii_digit() { 2 } else { 1 };
            screen.draw_text(row as usize, col, &ch.to_string(), attr);
            // Advance syntax state to keep it in sync with screen drawing.
            rs_syn_update(0);
            col += 1;
        }
    }
    // Return the last row used.  This simple implementation always draws a
    // single row, so return the provided row value.
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_line_with_highlight() {
        let mut sb = ScreenBuffer::new(20, 2);
        let line = std::ffi::CString::new("abc123").unwrap();
        let row = rs_draw_line(&mut sb, 0, line.as_ptr());
        assert_eq!(row, 0);
        // Digits should receive attribute 2.
        let text = sb.line_as_string(0);
        assert_eq!(&text[..6], "abc123");
        let diff = sb.flush_diff();
        assert_eq!(&diff[0].attrs[..6], &[1, 1, 1, 2, 2, 2]);
    }
}
