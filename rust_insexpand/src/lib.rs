use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn rs_insexpand_complete(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    let cstr = unsafe { CStr::from_ptr(input) };
    let mut owned = cstr.to_string_lossy().into_owned();
    owned.push_str("_completed");
    CString::new(owned).unwrap().into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn complete_appends_suffix() {
        let input = CString::new("hello").unwrap();
        let ptr = rs_insexpand_complete(input.as_ptr());
        let cstr = unsafe { CStr::from_ptr(ptr) };
        assert_eq!(cstr.to_str().unwrap(), "hello_completed");
    }
}
