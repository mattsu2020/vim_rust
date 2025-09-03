use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// Dummy bufwrite function exposed to C.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn bufwrite_dummy(path: *const c_char) -> c_int {
    if path.is_null() {
        return -1;
    }
    // Convert C string to Rust &str for demonstration purposes.
    let c_str = unsafe { CStr::from_ptr(path) };
    match c_str.to_str() {
        Ok(_s) => 0,
        Err(_) => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn dummy_ok() {
        let s = CString::new("test").unwrap();
        assert_eq!(bufwrite_dummy(s.as_ptr()), 0);
    }
}
