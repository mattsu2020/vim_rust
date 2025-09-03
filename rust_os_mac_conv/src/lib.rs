#![cfg(target_os = "macos")]

use std::ffi::CStr;
use std::os::raw::{c_int, c_char};
use std::ptr;

/// Simplified string conversion used on macOS.
/// Copies the input bytes and appends a NUL terminator.
#[no_mangle]
pub extern "C" fn mac_string_convert(
    ptr_in: *const c_char,
    len: c_int,
    lenp: *mut c_int,
    _fail_on_error: c_int,
    _from_enc: c_int,
    _to_enc: c_int,
    _unconvlenp: *mut c_int,
) -> *mut c_char {
    if ptr_in.is_null() || len < 0 {
        return ptr::null_mut();
    }
    unsafe {
        if !lenp.is_null() {
            *lenp = len;
        }
        let slice = std::slice::from_raw_parts(ptr_in as *const u8, len as usize);
        let mut vec = slice.to_vec();
        vec.push(0);
        let boxed = vec.into_boxed_slice();
        Box::into_raw(boxed) as *mut c_char
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn copies_string() {
        let src = CString::new("abc").unwrap();
        let mut out_len = 0;
        let out = unsafe {
            mac_string_convert(src.as_ptr(), 3, &mut out_len, 0, 0, 0, ptr::null_mut())
        };
        assert_eq!(out_len, 3);
        unsafe {
            let s = CStr::from_ptr(out).to_str().unwrap();
            assert_eq!(s, "abc");
            libc::free(out as *mut _);
        }
    }
}
