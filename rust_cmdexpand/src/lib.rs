use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Expand a command-line string.
///
/// Currently this is a placeholder that simply returns a copy of the input.
#[no_mangle]
pub extern "C" fn rs_cmd_expand(input: *const c_char) -> *const c_char {
    if input.is_null() {
        return std::ptr::null();
    }
    let c_str = unsafe { CStr::from_ptr(input) };
    let owned = match CString::new(c_str.to_string_lossy().into_owned()) {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
    };
    owned.into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;

    #[test]
    fn identity_expand() {
        let input = CString::new("foo").unwrap();
        let ptr = rs_cmd_expand(input.as_ptr());
        let out = unsafe { CStr::from_ptr(ptr) };
        assert_eq!(out.to_str().unwrap(), "foo");
        unsafe {
            drop(CString::from_raw(ptr as *mut c_char));
        }
    }
}
