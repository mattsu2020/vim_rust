use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Compute a simple indentation level based on unmatched '{' and '}' characters.
/// The result is the number of unmatched '{' times four.
#[no_mangle]
pub extern "C" fn rs_cindent_level(prev_line: *const c_char) -> c_int {
    if prev_line.is_null() {
        return 0;
    }
    let line = unsafe { CStr::from_ptr(prev_line) }.to_string_lossy();
    let mut level: i32 = 0;
    for ch in line.chars() {
        match ch {
            '{' => level += 1,
            '}' if level > 0 => level -= 1,
            _ => {}
        }
    }
    level * 4
}

/// Return a very naive completion by appending "_complete" to the given prefix.
/// The caller is responsible for freeing the returned string via `libc::free`.
#[no_mangle]
pub extern "C" fn rs_complete_word(prefix: *const c_char) -> *mut c_char {
    if prefix.is_null() {
        return std::ptr::null_mut();
    }
    let pref = unsafe { CStr::from_ptr(prefix) }.to_string_lossy();
    let completion = format!("{}{}", pref, "_complete");
    CString::new(completion).unwrap().into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indent_counts_braces() {
        let line = CString::new("if (x) {\n").unwrap();
        assert_eq!(rs_cindent_level(line.as_ptr()), 4);
        let closing = CString::new("}\n").unwrap();
        assert_eq!(rs_cindent_level(closing.as_ptr()), 0);
    }

    #[test]
    fn completion_appends_suffix() {
        let prefix = CString::new("foo").unwrap();
        let ptr = rs_complete_word(prefix.as_ptr());
        let s = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_string();
        unsafe { libc::free(ptr as *mut libc::c_void) };
        assert_eq!(s, "foo_complete");
    }
}
