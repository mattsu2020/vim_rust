use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libc::{c_double, strtod};

/// Parse a C string into a floating point value, similar to Vim's `string2float`.
///
/// Returns the number of bytes consumed from `text`.
#[no_mangle]
pub unsafe extern "C" fn string2float(text: *const u8, value: *mut c_double, _skip_quotes: c_int) -> c_int {
    if text.is_null() || value.is_null() {
        return 0;
    }

    let slice = CStr::from_ptr(text as *const c_char);
    let bytes = slice.to_bytes();

    // Handle special cases: "inf", "-inf", "nan"
    if bytes.starts_with(b"inf") {
        *value = f64::INFINITY;
        return 3;
    }
    if bytes.starts_with(b"-inf") {
        *value = f64::NEG_INFINITY;
        return 4;
    }
    if bytes.starts_with(b"nan") {
        *value = f64::NAN;
        return 3;
    }

    let mut end_ptr: *mut c_char = std::ptr::null_mut();
    let parsed = strtod(text as *const c_char, &mut end_ptr);
    *value = parsed as f64;

    end_ptr.offset_from(text as *const c_char) as c_int
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_number() {
        let input = std::ffi::CString::new("3.14").unwrap();
        let mut out = 0.0f64;
        let len = unsafe { string2float(input.as_ptr() as *const u8, &mut out, 0) };
        assert_eq!(len, 4); // "3.14"
        assert!((out - 3.14).abs() < 1e-6);
    }

    #[test]
    fn parse_inf() {
        let input = std::ffi::CString::new("inf").unwrap();
        let mut out = 0.0f64;
        let len = unsafe { string2float(input.as_ptr() as *const u8, &mut out, 0) };
        assert_eq!(len, 3);
        assert!(out.is_infinite() && out.is_sign_positive());
    }
}
