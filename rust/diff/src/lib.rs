use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Compute a unified diff of two C strings.
///
/// # Safety
/// The input pointers must be valid C strings or null.
#[no_mangle]
pub extern "C" fn diff_unified(a: *const c_char, b: *const c_char) -> *mut c_char {
    unsafe {
        let a_str = if a.is_null() {
            ""
        } else {
            CStr::from_ptr(a).to_str().unwrap_or("")
        };
        let b_str = if b.is_null() {
            ""
        } else {
            CStr::from_ptr(b).to_str().unwrap_or("")
        };
        let diff = similar::TextDiff::from_lines(a_str, b_str)
            .unified_diff()
            .to_string();
        CString::new(diff).unwrap().into_raw()
    }
}

/// Free a string returned by [`diff_unified`].
#[no_mangle]
pub extern "C" fn diff_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn diff_empty() {
        let a = CString::new("same\n").unwrap();
        let b = CString::new("same\n").unwrap();
        let res_ptr = diff_unified(a.as_ptr(), b.as_ptr());
        let res = unsafe { CStr::from_ptr(res_ptr).to_str().unwrap() };
        assert_eq!(res, "");
        diff_free(res_ptr);
    }

    #[test]
    fn diff_insert_delete() {
        let a = CString::new("a\nb\n").unwrap();
        let b = CString::new("a\nc\n").unwrap();
        let res_ptr = diff_unified(a.as_ptr(), b.as_ptr());
        let res = unsafe { CStr::from_ptr(res_ptr).to_str().unwrap().to_owned() };
        diff_free(res_ptr);
        assert!(res.contains("-b"));
        assert!(res.contains("+c"));
    }
}
