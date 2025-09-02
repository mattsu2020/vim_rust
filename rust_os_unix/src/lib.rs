use std::os::raw::c_char;
use libc;

// Change current working directory
#[no_mangle]
pub extern "C" fn rs_chdir(path: *const c_char) -> libc::c_int {
    if path.is_null() {
        return -1;
    }
    unsafe { libc::chdir(path) }
}

// Set environment variable
#[no_mangle]
pub extern "C" fn rs_setenv(name: *const c_char, value: *const c_char, overwrite: libc::c_int) -> libc::c_int {
    if name.is_null() || value.is_null() {
        return -1;
    }
    unsafe { libc::setenv(name, value, overwrite) }
}

// Unset environment variable
#[no_mangle]
pub extern "C" fn rs_unsetenv(name: *const c_char) -> libc::c_int {
    if name.is_null() {
        return -1;
    }
    unsafe { libc::unsetenv(name) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_env() {
        let name = CString::new("RUST_OS_UNIX_TEST").unwrap();
        let val = CString::new("VALUE").unwrap();
        assert_eq!(rs_setenv(name.as_ptr(), val.as_ptr(), 1), 0);
        unsafe {
            let got = libc::getenv(name.as_ptr());
            assert!(!got.is_null());
        }
        assert_eq!(rs_unsetenv(name.as_ptr()), 0);
        unsafe {
            let got = libc::getenv(name.as_ptr());
            assert!(got.is_null());
        }
    }

    #[test]
    fn test_chdir() {
        // Using /tmp or / as safe directories
        #[cfg(target_os = "windows")]
        let target = CString::new("\\").unwrap();
        #[cfg(not(target_os = "windows"))]
        let target = CString::new("/").unwrap();
        assert_eq!(rs_chdir(target.as_ptr()), 0);
    }
}
