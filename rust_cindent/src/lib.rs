use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_long};

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

fn is_funcdecl(src: &str) -> bool {
    let mut paren = 0;
    let mut saw_open = false;
    let mut last_close = None;
    for (i, ch) in src.char_indices() {
        match ch {
            '(' => {
                paren += 1;
                saw_open = true;
            }
            ')' => {
                if paren == 0 {
                    return false;
                }
                paren -= 1;
                if paren == 0 {
                    last_close = Some(i);
                }
            }
            ';' | '\'' | '"' if !saw_open => return false,
            _ => {}
        }
    }
    if !saw_open || paren != 0 {
        return false;
    }
    if let Some(idx) = last_close {
        let after = src[idx + 1..].trim();
        after.is_empty() || after.starts_with('{')
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn cin_isfuncdecl(
    sp: *mut *const c_char,
    _first_lnum: c_long,
    _min_lnum: c_long,
) -> c_int {
    if sp.is_null() {
        return 0;
    }
    let s_ptr = unsafe { *sp };
    if s_ptr.is_null() {
        return 0;
    }
    let c_str = unsafe { CStr::from_ptr(s_ptr) };
    let text = match c_str.to_str() {
        Ok(t) => t,
        Err(_) => return 0,
    };
    is_funcdecl(text) as c_int
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

    #[test]
    fn detect_funcdecl_single_line() {
        let s = CString::new("int foo(int a, int b)").unwrap();
        let mut ptr = s.as_ptr();
        assert_eq!(cin_isfuncdecl(&mut ptr, 1, 0), 1);
    }

    #[test]
    fn detect_funcdecl_multi_line() {
        let s = CString::new("int foo(\n    int a,\n    int b\n)").unwrap();
        let mut ptr = s.as_ptr();
        assert_eq!(cin_isfuncdecl(&mut ptr, 1, 0), 1);
    }

    #[test]
    fn reject_prototype() {
        let s = CString::new("int foo(int a);").unwrap();
        let mut ptr = s.as_ptr();
        assert_eq!(cin_isfuncdecl(&mut ptr, 1, 0), 0);
    }
}
