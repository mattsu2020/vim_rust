use libc::{c_char, c_int};
use std::ffi::CStr;
use std::ptr;

/// Safe representation of the Vim screen buffers.
pub struct ScreenBuffer {
    width: usize,
    height: usize,
    lines: Vec<char>,
    attrs: Vec<u8>,
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            lines: vec![' '; width * height],
            attrs: vec![0; width * height],
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

    pub fn line_as_string(&self, row: usize) -> String {
        if row >= self.height {
            return String::new();
        }
        self.lines[row * self.width..row * self.width + self.width]
            .iter()
            .collect()
    }
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
        unsafe { drop(Box::from_raw(buf)); }
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
}
