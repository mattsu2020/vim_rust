use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::io::{self, Write};
use crossterm::{execute, terminal::{enable_raw_mode, disable_raw_mode}, cursor};

/// Initialize the terminal for raw mode.  Returns 0 on success.
#[no_mangle]
pub extern "C" fn term_start() -> c_int {
    if enable_raw_mode().is_ok() {
        let _ = execute!(io::stdout(), cursor::Hide);
        0
    } else {
        -1
    }
}

/// Write a UTF-8 string to the terminal.  `ptr` must be a
/// NUL-terminated C string.  Returns 0 on success.
#[no_mangle]
pub extern "C" fn term_write(ptr: *const c_char) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let c_str = unsafe { CStr::from_ptr(ptr) };
    match c_str.to_str() {
        Ok(s) => {
            let mut out = io::stdout();
            if out.write_all(s.as_bytes()).is_ok() && out.flush().is_ok() {
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

/// Restore the terminal state.  Returns 0 on success.
#[no_mangle]
pub extern "C" fn term_stop() -> c_int {
    disable_raw_mode().map(|_| 0).unwrap_or(-1)
}
