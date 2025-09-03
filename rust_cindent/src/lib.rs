use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// A very simplified indentation computation.
/// Counts unmatched '{' and '}' to determine indentation level.
#[no_mangle]
pub extern "C" fn rs_cindent(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }
    let c_str = unsafe { CStr::from_ptr(line) };
    let mut level = 0;
    for ch in c_str.to_bytes() {
        match *ch as char {
            '{' => level += 1,
            '}' => if level > 0 { level -= 1; },
            _ => {}
        }
    }
    level as c_int
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn simple_braces() {
        let line = CString::new("if (x) {").unwrap();
        assert_eq!(rs_cindent(line.as_ptr()), 1);
    }

    #[test]
    fn closing_brace() {
        let line = CString::new("}").unwrap();
        assert_eq!(rs_cindent(line.as_ptr()), 0);
    }
}
