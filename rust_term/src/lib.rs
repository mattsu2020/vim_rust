use std::ffi::CStr;
use std::io::{stdout, Write};
use std::os::raw::{c_char, c_int};

mod platform;

/// Simple owned buffer used for terminal output.
struct TermBuffer {
    buf: Vec<u8>,
    cap: usize,
}

impl TermBuffer {
    fn new() -> Self {
        // use a small default similar to Vim's OUT_SIZE
        let cap = 1024;
        Self { buf: Vec::with_capacity(cap), cap }
    }

    fn write_byte(&mut self, b: u8) {
        self.buf.push(b);
        if self.buf.len() >= self.cap {
            // ignore errors on flush; caller can handle via explicit flush
            let _ = self.flush();
        }
    }

    fn write_str(&mut self, s: &str) {
        for &b in s.as_bytes() {
            self.write_byte(b);
        }
    }

    fn flush(&mut self) -> c_int {
        let mut out = stdout();
        if out.write_all(&self.buf).is_ok() && out.flush().is_ok() {
            self.buf.clear();
            0
        } else {
            -1
        }
    }
}

/// Terminal state object holding the output buffer.
pub struct Terminal {
    buffer: TermBuffer,
}

impl Terminal {
    fn new() -> Self {
        Self { buffer: TermBuffer::new() }
    }

    fn move_cursor(&mut self, x: c_int, y: c_int) -> c_int {
        self.buffer
            .write_str(&format!("\x1B[{};{}H", y + 1, x + 1));
        self.buffer.flush()
    }

    fn clear_screen(&mut self) -> c_int {
        self.buffer.write_str("\x1B[2J");
        self.buffer.flush()
    }

    fn print(&mut self, s: &str) -> c_int {
        self.buffer.write_str(s);
        self.buffer.flush()
    }
}

// --- FFI exposed functions -------------------------------------------------

#[no_mangle]
pub extern "C" fn rust_term_new() -> *mut Terminal {
    Box::into_raw(Box::new(Terminal::new()))
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_free(term: *mut Terminal) {
    if !term.is_null() {
        drop(Box::from_raw(term));
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_out_char(term: *mut Terminal, c: c_int) -> c_int {
    if term.is_null() {
        return -1;
    }
    let term = &mut *term;
    term.buffer.write_byte(c as u8);
    0
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_out_flush(term: *mut Terminal) -> c_int {
    if term.is_null() {
        return -1;
    }
    (&mut *term).buffer.flush()
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_move_cursor(
    term: *mut Terminal,
    x: c_int,
    y: c_int,
) -> c_int {
    if term.is_null() {
        return -1;
    }
    (&mut *term).move_cursor(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_clear_screen(term: *mut Terminal) -> c_int {
    if term.is_null() {
        return -1;
    }
    (&mut *term).clear_screen()
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_print(
    term: *mut Terminal,
    s: *const c_char,
) -> c_int {
    if term.is_null() || s.is_null() {
        return -1;
    }
    let term = &mut *term;
    let c_str = CStr::from_ptr(s);
    match c_str.to_str() {
        Ok(s) => term.print(s),
        Err(_) => -1,
    }
}

/// Returns the number of colors supported by the terminal.
#[no_mangle]
pub extern "C" fn rust_term_color_count() -> c_int {
    8
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_get_winsize(
    width: *mut c_int,
    height: *mut c_int,
) -> c_int {
    platform::get_winsize(width, height)
}

// --- Tests -----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn basic_output() {
        let term = unsafe { rust_term_new() };
        assert!(!term.is_null());
        unsafe {
            assert_eq!(rust_term_move_cursor(term, 0, 0), 0);
            assert_eq!(rust_term_clear_screen(term), 0);
            let s = CString::new("test").unwrap();
            assert_eq!(rust_term_print(term, s.as_ptr()), 0);
            assert_eq!(rust_term_out_char(term, b'\n' as c_int), 0);
            assert_eq!(rust_term_out_flush(term), 0);
            rust_term_free(term);
        }
    }

    #[cfg(unix)]
    #[test]
    fn get_winsize() {
        let mut w = 0;
        let mut h = 0;
        unsafe {
            // Might return zero when stdout is not a TTY; ensure no crash.
            let _ = rust_term_get_winsize(&mut w, &mut h);
        }
    }
}

