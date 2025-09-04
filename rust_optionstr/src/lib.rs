use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use rust_option::rs_parse_option;

pub fn apply_option_str(s: &str) -> bool {
    for part in s.split_whitespace() {
        let c = match CString::new(part) {
            Ok(c) => c,
            Err(_) => return false,
        };
        if !rs_parse_option(c.as_ptr()) {
            return false;
        }
    }
    true
}

#[no_mangle]
pub extern "C" fn rs_apply_option_str(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(s) };
    let Ok(s) = c_str.to_str() else { return false };
    apply_option_str(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_option::{rs_get_option, rs_options_init};
    use std::ffi::CString;

    #[test]
    fn parse_multiple_options() {
        rs_options_init();
        assert!(apply_option_str("background=dark fileformat=dos"));
        let name = CString::new("background").unwrap();
        let val_ptr = rs_get_option(name.as_ptr());
        assert!(!val_ptr.is_null());
        let val = unsafe { CString::from_raw(val_ptr) };
        assert_eq!(val.to_str().unwrap(), "dark");
    }
}

