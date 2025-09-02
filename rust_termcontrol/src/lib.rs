use std::ffi::CStr;
use std::io::{stdout, Write};
use std::os::raw::{c_char, c_int};

use crossterm::style::available_color_count;
use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

/// Returns the number of colors supported by the terminal.
#[no_mangle]
pub extern "C" fn rust_termcontrol_color_count() -> c_int {
    available_color_count() as c_int
}

/// Move the cursor to the given position.
#[no_mangle]
pub extern "C" fn rust_termcontrol_move_cursor(x: c_int, y: c_int) -> c_int {
    let mut out = stdout();
    match out.execute(MoveTo(x as u16, y as u16)) {
        Ok(_) => match out.flush() {
            Ok(_) => 0,
            Err(_) => -1,
        },
        Err(_) => -1,
    }
}

/// Clear the entire screen.
#[no_mangle]
pub extern "C" fn rust_termcontrol_clear_screen() -> c_int {
    let mut out = stdout();
    match out.execute(Clear(ClearType::All)) {
        Ok(_) => match out.flush() {
            Ok(_) => 0,
            Err(_) => -1,
        },
        Err(_) => -1,
    }
}

/// Print a string to the terminal.
#[no_mangle]
pub unsafe extern "C" fn rust_termcontrol_print(s: *const c_char) -> c_int {
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
        // Exercise basic functionality. These calls should not panic even
        // when no real terminal is attached.
        let _ = rust_termcontrol_color_count();
        let _ = rust_termcontrol_move_cursor(0, 0);
        let _ = rust_termcontrol_clear_screen();
        let c_string = std::ffi::CString::new("test").unwrap();
        let _ = unsafe { rust_termcontrol_print(c_string.as_ptr()) };
    }
}
