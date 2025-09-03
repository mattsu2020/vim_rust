use libc::{c_char, c_int};
use std::ffi::{CStr, CString};
use std::ptr;

/// Safe representation of the Vim screen buffers.
pub struct ScreenBuffer {
    width: usize,
    height: usize,
    lines: Vec<char>,
    attrs: Vec<u8>,
    /// Previous frame used for diffing.
    prev_lines: Vec<char>,
    prev_attrs: Vec<u8>,
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            lines: vec![' '; width * height],
            attrs: vec![0; width * height],
            prev_lines: vec![' '; width * height],
            prev_attrs: vec![0; width * height],
        }
    }

    pub fn draw_text(&mut self, row: usize, col: usize, text: &str, attr: u8) {
        for (i, ch) in text.chars().enumerate() {
            if row >= self.height || col + i >= self.width {
                break;
            }
            let idx = row * self.width + col + i;
            self.lines[idx] = ch;
            self.attrs[idx] = attr;
        }
    }

    pub fn clear_line(&mut self, row: usize, attr: u8) {
        if row >= self.height {
            return;
        }
        let start = row * self.width;
        for i in 0..self.width {
            self.lines[start + i] = ' ';
            self.attrs[start + i] = attr;
        }
    }

    /// Clear the entire screen buffer with the given attribute.
    pub fn clear(&mut self, attr: u8) {
        for i in 0..self.lines.len() {
            self.lines[i] = ' ';
            self.attrs[i] = attr;
        }
    }

    /// Apply a highlight attribute to a range without modifying the text.
    pub fn highlight_range(&mut self, row: usize, col: usize, len: usize, attr: u8) {
        if row >= self.height {
            return;
        }
        for i in 0..len {
            if col + i >= self.width {
                break;
            }
            let idx = row * self.width + col + i;
            self.attrs[idx] = attr;
        }
    }

    pub fn line_as_string(&self, row: usize) -> String {
        if row >= self.height {
            return String::new();
        }
        self.lines[row * self.width..row * self.width + self.width]
            .iter()
            .collect()
    }

    /// Compute the difference with the previous frame.
    pub fn flush_diff(&mut self) -> Vec<LineDiff> {
        let mut diffs = Vec::new();
        for row in 0..self.height {
            let start = row * self.width;
            let end = start + self.width;
            if self.lines[start..end] != self.prev_lines[start..end]
                || self.attrs[start..end] != self.prev_attrs[start..end]
            {
                let text: String = self.lines[start..end].iter().collect();
                let attrs = self.attrs[start..end].to_vec();
                self.prev_lines[start..end].copy_from_slice(&self.lines[start..end]);
                self.prev_attrs[start..end].copy_from_slice(&self.attrs[start..end]);
                diffs.push(LineDiff { row, text, attrs });
            }
        }
        diffs
    }
}

/// A single line update returned by [`ScreenBuffer::flush_diff`].
pub struct LineDiff {
    pub row: usize,
    pub text: String,
    pub attrs: Vec<u8>,
}

#[no_mangle]
pub extern "C" fn rs_screen_new(width: c_int, height: c_int) -> *mut ScreenBuffer {
    if width <= 0 || height <= 0 {
        return ptr::null_mut();
    }
    Box::into_raw(Box::new(ScreenBuffer::new(width as usize, height as usize)))
}

#[no_mangle]
pub extern "C" fn rs_screen_free(buf: *mut ScreenBuffer) {
    if !buf.is_null() {
        unsafe {
            drop(Box::from_raw(buf));
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_screen_draw_text(
    buf: *mut ScreenBuffer,
    row: c_int,
    col: c_int,
    text: *const c_char,
    attr: u8,
) {
    if buf.is_null() || text.is_null() {
        return;
    }
    let screen = unsafe { &mut *buf };
    let c_str = unsafe { CStr::from_ptr(text) };
    if let Ok(s) = c_str.to_str() {
        screen.draw_text(row as usize, col as usize, s, attr);
    }
}

#[no_mangle]
pub extern "C" fn rs_screen_clear_line(buf: *mut ScreenBuffer, row: c_int, attr: u8) {
    if buf.is_null() {
        return;
    }
    let screen = unsafe { &mut *buf };
    screen.clear_line(row as usize, attr);
}

#[no_mangle]
pub extern "C" fn rs_screen_clear(buf: *mut ScreenBuffer, attr: u8) {
    if buf.is_null() {
        return;
    }
    let screen = unsafe { &mut *buf };
    screen.clear(attr);
}

#[no_mangle]
pub extern "C" fn rs_screen_highlight(
    buf: *mut ScreenBuffer,
    row: c_int,
    col: c_int,
    len: c_int,
    attr: u8,
) {
    if buf.is_null() {
        return;
    }
    let screen = unsafe { &mut *buf };
    screen.highlight_range(row as usize, col as usize, len as usize, attr);
}

/// Callback used by [`rs_screen_flush`].
pub type FlushCallback =
    extern "C" fn(row: c_int, text: *const c_char, attr: *const u8, len: c_int);

#[no_mangle]
pub extern "C" fn rs_screen_flush(buf: *mut ScreenBuffer, cb: Option<FlushCallback>) {
    if buf.is_null() {
        return;
    }
    if let Some(callback) = cb {
        let screen = unsafe { &mut *buf };
        let empty = CString::new("").unwrap();
        for diff in screen.flush_diff() {
            match CString::new(diff.text) {
                Ok(c_text) => {
                    // Callback is expected to copy the data immediately.
                    callback(
                        diff.row as c_int,
                        c_text.as_ptr(),
                        diff.attrs.as_ptr(),
                        diff.attrs.len() as c_int,
                    );
                }
                Err(_) => {
                    // Text contained an interior NUL; report empty line.
                    callback(diff.row as c_int, empty.as_ptr(), diff.attrs.as_ptr(), 0);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::sync::Mutex;
    use std::time::Instant;

    static CALLBACK_DATA: Mutex<(i32, String)> = Mutex::new((0, String::new()));

    extern "C" fn capture_cb(_row: c_int, text: *const c_char, _attr: *const u8, len: c_int) {
        let s = unsafe { CStr::from_ptr(text).to_str().unwrap_or("") };
        let mut data = CALLBACK_DATA.lock().unwrap();
        data.0 = len;
        data.1 = s.to_string();
    }

    #[test]
    fn draw_and_clear() {
        let mut sb = ScreenBuffer::new(10, 2);
        sb.draw_text(0, 0, "hi", 1);
        assert_eq!(sb.line_as_string(0), "hi        ");
        sb.clear_line(0, 0);
        assert_eq!(sb.line_as_string(0), "          ");
    }

    #[test]
    fn perf_draw() {
        let mut sb = ScreenBuffer::new(80, 24);
        let start = Instant::now();
        for _ in 0..1000 {
            sb.draw_text(0, 0, "hello world", 1);
        }
        let elapsed = start.elapsed();
        // Print timing for performance measurement
        eprintln!("draw_text 1000x took {:?}", elapsed);
        assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn diff_terminal() {
        let mut sb = ScreenBuffer::new(5, 2);
        sb.draw_text(0, 0, "abc", 1);
        let diff = sb.flush_diff();
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].row, 0);
        assert_eq!(diff[0].text, "abc  ");
        sb.draw_text(0, 1, "d", 1);
        let diff = sb.flush_diff();
        assert_eq!(diff[0].text, "adc  ");
    }

    #[test]
    fn diff_gui_attr() {
        let mut sb = ScreenBuffer::new(5, 1);
        sb.draw_text(0, 0, "ab", 1);
        sb.flush_diff(); // consume
        sb.draw_text(0, 0, "ab", 2); // change attrs only
        let diff = sb.flush_diff();
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].attrs[..2], [2, 2]);
    }

    #[test]
    fn highlight_range_diff() {
        let mut sb = ScreenBuffer::new(6, 1);
        sb.draw_text(0, 0, "abcdef", 1);
        sb.flush_diff(); // baseline
        sb.highlight_range(0, 2, 3, 3);
        let diff = sb.flush_diff();
        assert_eq!(diff.len(), 1);
        // characters remain the same
        assert_eq!(diff[0].text, "abcdef");
        // highlight attr applied to cde
        assert_eq!(diff[0].attrs[2..5], [3, 3, 3]);
    }

    #[test]
    fn clear_whole_screen() {
        let mut sb = ScreenBuffer::new(3, 2);
        sb.draw_text(0, 0, "ab", 1);
        sb.draw_text(1, 0, "cd", 1);
        rs_screen_clear(&mut sb as *mut ScreenBuffer, 0);
        assert_eq!(sb.line_as_string(0), "   ");
        assert_eq!(sb.line_as_string(1), "   ");
    }

    #[test]
    fn flush_handles_null_byte() {
        let mut sb = ScreenBuffer::new(5, 1);
        sb.draw_text(0, 0, "a\0b", 1);
        rs_screen_flush(&mut sb as *mut ScreenBuffer, Some(capture_cb));
        let data = CALLBACK_DATA.lock().unwrap();
        assert_eq!(data.0, 0);
        assert_eq!(data.1, "");
    }
}
