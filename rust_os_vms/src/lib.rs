#![cfg(target_os = "vms")]

use libc::{c_char, c_int};
use std::ffi::{CStr, CString};

#[no_mangle]
pub extern "C" fn os_vms_startup() {
    // Placeholder for VMS specific initialization.
}

#[no_mangle]
pub extern "C" fn os_vms_shutdown() {
    // Placeholder for VMS specific shutdown.
}

/// Convert a path from Vim's internal representation to VMS style.
#[no_mangle]
pub extern "C" fn os_vms_path_from_vim(path: *const c_char) -> *mut c_char {
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
pub extern "C" fn os_vms_isatty(fd: c_int) -> c_int {
    unsafe { libc::isatty(fd) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn test_path_roundtrip() {
        let input = CString::new("disk:[dir]file.txt").unwrap();
        let out_ptr = os_vms_path_from_vim(input.as_ptr());
        unsafe {
            let out = CStr::from_ptr(out_ptr).to_string_lossy().into_owned();
            libc::free(out_ptr as *mut _);
            assert_eq!(out, "disk:[dir]file.txt");
        }
    }
}
