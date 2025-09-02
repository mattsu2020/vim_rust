use std::os::raw::{c_char, c_int, c_void};

/// Check if an autocmd for `event` exists. Currently this is a stub
/// implementation that always returns 0 (false).
#[no_mangle]
pub extern "C" fn rs_has_autocmd(
    _event: c_int,
    _sfname: *const c_char,
    _buf: *mut c_void,
) -> c_int {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn has_autocmd_always_false() {
        let name = CString::new("file.txt").unwrap();
        let res = unsafe { rs_has_autocmd(0, name.as_ptr(), std::ptr::null_mut()) };
        assert_eq!(res, 0);
    }
}
