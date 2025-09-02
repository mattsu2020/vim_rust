use std::ffi::CStr;
use std::io::{stdout, Write};
use std::os::raw::{c_char, c_int};

#[cfg(unix)]
use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};

#[cfg(unix)]
use termios::Termios;

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

fn write_esc(seq: &str) -> c_int {
    #[cfg(unix)]
    {
        let _ = Termios::from_fd(STDOUT_FILENO);
    }
    let mut out = stdout();
    if out.write_all(seq.as_bytes()).is_ok() && out.flush().is_ok() {
        0
    } else {
        -1
    }
}

/// Returns the number of colors supported by the terminal.
#[no_mangle]
pub extern "C" fn rust_term_color_count() -> c_int {
    8
}

/// Move the cursor to the given position.
#[no_mangle]
pub extern "C" fn rust_term_move_cursor(x: c_int, y: c_int) -> c_int {
    write_esc(&format!("\x1B[{};{}H", y + 1, x + 1))
}

/// Clear the entire screen.
#[no_mangle]
pub extern "C" fn rust_term_clear_screen() -> c_int {
    write_esc("\x1B[2J")
}

/// Print a string to the terminal.
#[no_mangle]
pub unsafe extern "C" fn rust_term_print(s: *const c_char) -> c_int {
    if s.is_null() {
        return -1;
    }
    let c_str = CStr::from_ptr(s);
    if let Ok(str_slice) = c_str.to_str() {
        let mut out = stdout();
        if out.write_all(str_slice.as_bytes()).is_ok() && out.flush().is_ok() {
            str_slice.len() as c_int
        } else {
            -1
        }
    } else {
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_functions() {
        let mut w = 0;
        let mut h = 0;
        unsafe {
            let _ = rust_term_get_winsize(&mut w, &mut h);
        }
        let _ = rust_term_color_count();
        let _ = rust_term_move_cursor(0, 0);
        let _ = rust_term_clear_screen();
        let c_string = std::ffi::CString::new("test").unwrap();
        let _ = unsafe { rust_term_print(c_string.as_ptr()) };
    }
}

