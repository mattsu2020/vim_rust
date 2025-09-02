use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use similar::{Algorithm, TextDiff};

fn diff_internal(a: &str, b: &str) -> String {
    TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .diff_lines(a, b)
        .unified_diff()
        .to_string()
}

#[no_mangle]
pub extern "C" fn vim_xdiff_diff(a: *const c_char, b: *const c_char) -> *mut c_char {
    if a.is_null() || b.is_null() {
        return std::ptr::null_mut();
    }
    let a_str = unsafe { CStr::from_ptr(a) }.to_string_lossy().into_owned();
    let b_str = unsafe { CStr::from_ptr(b) }.to_string_lossy().into_owned();
    let result = diff_internal(&a_str, &b_str);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn vim_xdiff_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr)); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ffi::CStr;
    use std::time::Instant;

    #[test]
    fn diff_basic() {
        let a = "a\nb\n";
        let b = "a\nc\n";
        let diff = diff_internal(a, b);
        assert!(diff.contains("-b"));
        assert!(diff.contains("+c"));
    }

    #[test]
    fn ffi_roundtrip() {
        let a = CString::new("one\ntwo\n").unwrap();
        let b = CString::new("one\nthree\n").unwrap();
        let ptr = vim_xdiff_diff(a.as_ptr(), b.as_ptr());
        let diff = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        vim_xdiff_free(ptr);
        assert!(diff.contains("-two"));
        assert!(diff.contains("+three"));
    }

    #[test]
    fn performance_small() {
        let a = "a\n".repeat(1000);
        let mut b = "a\n".repeat(999);
        b.push_str("b\n");
        let start = Instant::now();
        let _ = diff_internal(&a, &b);
        // ensure the diff runs quickly for small inputs
        assert!(start.elapsed().as_millis() < 200);
    }
}
