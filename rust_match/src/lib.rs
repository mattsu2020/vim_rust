use regex::Regex;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

#[no_mangle]
pub extern "C" fn regex_new(pattern: *const c_char) -> *mut Regex {
    if pattern.is_null() {
        return ptr::null_mut();
    }
    let cstr = unsafe { CStr::from_ptr(pattern) };
    match Regex::new(cstr.to_str().unwrap_or("")) {
        Ok(re) => Box::into_raw(Box::new(re)),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn regex_is_match(re_ptr: *const Regex, text: *const c_char) -> bool {
    if re_ptr.is_null() || text.is_null() {
        return false;
    }
    let re = unsafe { &*re_ptr };
    let cstr = unsafe { CStr::from_ptr(text) };
    re.is_match(cstr.to_str().unwrap_or(""))
}

#[no_mangle]
pub extern "C" fn regex_free(re_ptr: *mut Regex) {
    if re_ptr.is_null() {
        return;
    }
    unsafe { drop(Box::from_raw(re_ptr)) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn basic_match() {
        let pat = CString::new("a+\\d").unwrap();
        let re = regex_new(pat.as_ptr());
        assert!(!re.is_null());
        let text = CString::new("aa1").unwrap();
        assert!(regex_is_match(re, text.as_ptr()));
        regex_free(re);
    }
}
