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

    /// Helper to iterate over a range of cells on a single row.
    /// Performs bounds checks for `row` and `col` and limits iteration to the
    /// current line.
    fn for_each_cell<F>(&mut self, row: usize, col: usize, len: usize, mut f: F)
    where
        F: FnMut(&mut Self, usize),
    {
        if row >= self.height || col >= self.width {
            return;
        }
        let start = row * self.width + col;
        let end = start + len.min(self.width - col);
        for idx in start..end {
            f(self, idx);
        }
    }

    pub fn clear_line(&mut self, row: usize, attr: u8) {
        self.for_each_cell(row, 0, self.width, |sb, idx| {
            sb.lines[idx] = ' ';
            sb.attrs[idx] = attr;
        });
    }

    /// Clear the entire screen buffer with the given attribute.
    pub fn clear(&mut self, attr: u8) {
        for row in 0..self.height {
            self.for_each_cell(row, 0, self.width, |sb, idx| {
                sb.lines[idx] = ' ';
                sb.attrs[idx] = attr;
            });
        }
    }

    /// Apply a highlight attribute to a range without modifying the text.
    pub fn highlight_range(&mut self, row: usize, col: usize, len: usize, attr: u8) {
        self.for_each_cell(row, col, len, |sb, idx| {
            sb.attrs[idx] = attr;
        });
    }

    /// Format `text` to fit within `width` cells.
    /// If `text` is shorter than `width` it is padded with spaces, and if it is
    /// longer it will be truncated.
    pub fn format_text(text: &str, width: usize) -> String {
        let mut s: String = text.chars().take(width).collect();
        if s.len() < width {
            s.push_str(&" ".repeat(width - s.len()));
        }
        s
    }

    /// Draw `text` after formatting it to `width` cells.
    pub fn draw_formatted_text(
        &mut self,
        row: usize,
        col: usize,
        text: &str,
        width: usize,
        attr: u8,
    ) {
        let formatted = Self::format_text(text, width);
        self.draw_text(row, col, &formatted, attr);
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
pub extern "C" fn rs_screen_draw_formatted(
    buf: *mut ScreenBuffer,
    row: c_int,
    col: c_int,
    text: *const c_char,
    width: c_int,
    attr: u8,
) {
    if buf.is_null() || text.is_null() {
        return;
    }
    let screen = unsafe { &mut *buf };
    let c_str = unsafe { CStr::from_ptr(text) };
    if let Ok(s) = c_str.to_str() {
        screen.draw_formatted_text(row as usize, col as usize, s, width as usize, attr);
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
        for diff in screen.flush_diff() {
            let c_text = CString::new(diff.text).unwrap();
            // Callback is expected to copy the data immediately.
            callback(
                diff.row as c_int,
                c_text.as_ptr(),
                diff.attrs.as_ptr(),
                diff.attrs.len() as c_int,
            );
        }
    }
}

// The screen buffer is manipulated from C and Rust code but not shared across
// threads, mark it as Send and Sync so it can be stored in static globals.
unsafe impl Send for ScreenBuffer {}
unsafe impl Sync for ScreenBuffer {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

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
    fn for_each_cell_bounds() {
        let mut sb = ScreenBuffer::new(5, 2);
        let mut idxs = Vec::new();
        sb.for_each_cell(1, 3, 10, |_, idx| idxs.push(idx));
        assert_eq!(idxs, vec![8, 9]);
        sb.for_each_cell(2, 0, 1, |_, idx| idxs.push(idx));
        sb.for_each_cell(0, 5, 1, |_, idx| idxs.push(idx));
        assert_eq!(idxs, vec![8, 9]);
    }

    #[test]
    fn format_and_draw() {
        let mut sb = ScreenBuffer::new(5, 1);
        let formatted = ScreenBuffer::format_text("ab", 4);
        assert_eq!(formatted, "ab  ");
        sb.draw_formatted_text(0, 0, "hello", 3, 1);
        assert_eq!(sb.line_as_string(0), "hel  ");
    }
}
