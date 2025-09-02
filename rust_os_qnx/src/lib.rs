#![cfg(target_os = "qnx")]

use libc::{c_char, c_int};
use std::ffi::{CStr, CString};

#[no_mangle]
pub extern "C" fn os_qnx_startup() {
    // Placeholder for QNX specific initialization.
}

#[no_mangle]
pub extern "C" fn os_qnx_shutdown() {
    // Placeholder for QNX specific shutdown.
}

/// Convert a path from Vim's internal representation to QNX style.
#[no_mangle]
pub extern "C" fn os_qnx_path_from_vim(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let s = CStr::from_ptr(path);
        CString::new(s.to_bytes()).unwrap().into_raw()
    }
}

/// Console helper that wraps libc `isatty`.
#[no_mangle]
pub extern "C" fn os_qnx_isatty(fd: c_int) -> c_int {
    unsafe { libc::isatty(fd) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn test_path_roundtrip() {
        let input = CString::new("/home/qnx/file").unwrap();
        let out_ptr = os_qnx_path_from_vim(input.as_ptr());
        unsafe {
            let out = CStr::from_ptr(out_ptr).to_string_lossy().into_owned();
            libc::free(out_ptr as *mut _);
            assert_eq!(out, "/home/qnx/file");
        }
    }
}
