#![cfg(target_os = "amiga")]

use libc::{c_char, c_int};
use std::ffi::{CStr, CString};

/// Perform any Amiga specific startup tasks.
#[no_mangle]
pub extern "C" fn os_amiga_startup() {
    // No-op for placeholder implementation.
}

/// Perform any Amiga specific shutdown tasks.
#[no_mangle]
pub extern "C" fn os_amiga_shutdown() {
    // No-op for placeholder implementation.
}

/// Convert a path from Vim's internal representation to an Amiga style path.
/// Currently this simply clones the input string.
#[no_mangle]
pub extern "C" fn os_amiga_path_from_vim(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let s = CStr::from_ptr(path);
        CString::new(s.to_bytes()).unwrap().into_raw()
    }
}

/// Simple console check using libc's `isatty`.
#[no_mangle]
pub extern "C" fn os_amiga_isatty(fd: c_int) -> c_int {
    unsafe { libc::isatty(fd) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn test_path_roundtrip() {
        let input = CString::new("ram:foo/bar").unwrap();
        let out_ptr = os_amiga_path_from_vim(input.as_ptr());
        unsafe {
            let out = CStr::from_ptr(out_ptr).to_string_lossy().into_owned();
            libc::free(out_ptr as *mut _);
            assert_eq!(out, "ram:foo/bar");
        }
    }
}
