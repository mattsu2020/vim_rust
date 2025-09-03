use std::os::raw::{c_char, c_int, c_void};

#[no_mangle]
pub extern "C" fn rs_sort_func_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    unsafe {
        let p1 = *(s1 as *const *const c_char);
        let p2 = *(s2 as *const *const c_char);
        if *p1 != b'<' as c_char && *p2 == b'<' as c_char {
            return -1;
        }
        if *p1 == b'<' as c_char && *p2 != b'<' as c_char {
            return 1;
        }
        libc::strcmp(p1, p2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn second_snr_returns_negative() {
        let a = CString::new("abc").unwrap();
        let b = CString::new("<snr>").unwrap();
        let res = unsafe {
            rs_sort_func_compare(
                &a.as_ptr() as *const _ as *const c_void,
                &b.as_ptr() as *const _ as *const c_void,
            )
        };
        assert!(res < 0);
    }

    #[test]
    fn first_snr_returns_positive() {
        let a = CString::new("<snr>").unwrap();
        let b = CString::new("abc").unwrap();
        let res = unsafe {
            rs_sort_func_compare(
                &a.as_ptr() as *const _ as *const c_void,
                &b.as_ptr() as *const _ as *const c_void,
            )
        };
        assert!(res > 0);
    }

    #[test]
    fn compare_uses_strcmp() {
        let a = CString::new("abc").unwrap();
        let b = CString::new("abd").unwrap();
        let res = unsafe {
            rs_sort_func_compare(
                &a.as_ptr() as *const _ as *const c_void,
                &b.as_ptr() as *const _ as *const c_void,
            )
        };
        assert!(res < 0);
    }
}
