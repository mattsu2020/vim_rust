use std::ffi::CStr;
use std::io::{stdout, Stdout, Write};
use std::os::raw::{c_char, c_int};

use crossterm::{cursor, execute, queue, style::Print, terminal};

/// Terminal state object wrapping a stdout handle.
pub struct Terminal {
    out: Stdout,
}

impl Terminal {
    fn new() -> Self {
        Self { out: stdout() }
    }

    fn move_cursor(&mut self, x: c_int, y: c_int) -> c_int {
        if execute!(self.out, cursor::MoveTo(x as u16, y as u16)).is_ok() {
            0
        } else {
            -1
        }
    }

    fn clear_screen(&mut self) -> c_int {
        if execute!(self.out, terminal::Clear(terminal::ClearType::All)).is_ok() {
            0
        } else {
            -1
        }
    }

    fn out_char(&mut self, c: c_int) -> c_int {
        let ch = (c as u8) as char;
        if queue!(self.out, Print(ch)).is_ok() {
            0
        } else {
            -1
        }
    }

    fn print(&mut self, s: &str) -> c_int {
        if execute!(self.out, Print(s)).is_ok() {
            0
        } else {
            -1
        }
    }

    fn flush(&mut self) -> c_int {
        if self.out.flush().is_ok() {
            0
        } else {
            -1
        }
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
    (&mut *term).out_char(c)
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_out_flush(term: *mut Terminal) -> c_int {
    if term.is_null() {
        return -1;
    }
    (&mut *term).flush()
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
    match terminal::size() {
        Ok((w, h)) => {
            if !width.is_null() {
                *width = w as c_int;
            }
            if !height.is_null() {
                *height = h as c_int;
            }
            0
        }
        Err(_) => -1,
    }
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

