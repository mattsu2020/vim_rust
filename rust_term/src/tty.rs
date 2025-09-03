use std::ffi::CStr;
use std::io::{stdout, Stdout, Write};
use std::os::raw::{c_char, c_int};

#[cfg(unix)]
use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};

use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor::MoveTo, QueueableCommand};

/// Terminal state object wrapping a `Stdout` handle.
pub struct Terminal {
    stdout: Stdout,
}

impl Terminal {
    fn new() -> Self {
        Self { stdout: stdout() }
    }

    fn move_cursor(&mut self, x: c_int, y: c_int) -> c_int {
        if self.stdout.queue(MoveTo(x as u16, y as u16)).is_ok()
            && self.stdout.flush().is_ok()
        {
            0
        } else {
            -1
        }
    }

    fn clear_screen(&mut self) -> c_int {
        if self.stdout.queue(Clear(ClearType::All)).is_ok() && self.stdout.flush().is_ok() {
            0
        } else {
            -1
        }
    }

    fn print(&mut self, s: &str) -> c_int {
        if self.stdout.queue(Print(s)).is_ok() && self.stdout.flush().is_ok() {
            0
        } else {
            -1
        }
    }

    fn out_char(&mut self, c: u8) -> c_int {
        if self.stdout.queue(Print(c as char)).is_ok() {
            0
        } else {
            -1
        }
    }

    fn out_flush(&mut self) -> c_int {
        self.stdout.flush().map(|_| 0).unwrap_or(-1)
    }
}

// --- FFI exposed functions -------------------------------------------------

#[no_mangle]
pub extern "C" fn rust_term_new() -> *mut Terminal {
    Box::into_raw(Box::new(Terminal::new()))
}

macro_rules! ffi_term {
    ($term:expr) => {
        ffi_term!($term, -1)
    };
    ($term:expr, $ret:expr) => {{
        if $term.is_null() {
            return $ret;
        }
        unsafe { &mut *$term }
    }};
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_free(term: *mut Terminal) {
    let term_ref = ffi_term!(term, ());
    drop(Box::from_raw(term_ref as *mut Terminal));
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_out_char(term: *mut Terminal, c: c_int) -> c_int {
    let term = ffi_term!(term);
    term.out_char(c as u8)
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_out_flush(term: *mut Terminal) -> c_int {
    let term = ffi_term!(term);
    term.out_flush()
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_move_cursor(term: *mut Terminal, x: c_int, y: c_int) -> c_int {
    let term = ffi_term!(term);
    term.move_cursor(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_clear_screen(term: *mut Terminal) -> c_int {
    let term = ffi_term!(term);
    term.clear_screen()
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_print(term: *mut Terminal, s: *const c_char) -> c_int {
    if s.is_null() {
        return -1;
    }
    let term = ffi_term!(term);
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
pub unsafe extern "C" fn rust_term_get_winsize(width: *mut c_int, height: *mut c_int) -> c_int {
    #[cfg(unix)]
    {
        let mut ws: winsize = std::mem::zeroed();
        if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) == -1 {
            return -1;
        }
        if !width.is_null() {
            *width = ws.ws_col as c_int;
        }
        if !height.is_null() {
            *height = ws.ws_row as c_int;
        }
        0
    }
    #[cfg(not(unix))]
    {
        -1
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

