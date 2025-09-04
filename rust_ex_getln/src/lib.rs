use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub fn get_line(script: &str, line_no: usize) -> Option<String> {
    script.lines().nth(line_no).map(|s| s.to_string())
}

#[no_mangle]
pub extern "C" fn rs_ex_getln(script: *const c_char, line_no: usize) -> *mut c_char {
    if script.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(script) };
    let Ok(script) = c_str.to_str() else { return std::ptr::null_mut() };
    match get_line(script, line_no) {
        Some(line) => CString::new(line).unwrap().into_raw(),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rs_ex_getln_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_are_returned_in_order() {
        let script = "cmd1\ncmd2";
        assert_eq!(get_line(script, 0).unwrap(), "cmd1");
        assert_eq!(get_line(script, 1).unwrap(), "cmd2");
        assert!(get_line(script, 2).is_none());
    }
}

