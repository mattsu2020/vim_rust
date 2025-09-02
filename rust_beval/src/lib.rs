use libc::{c_int, c_long, c_uchar, c_void};

#[no_mangle]
pub extern "C" fn find_word_under_cursor(
    _mouserow: c_int,
    _mousecol: c_int,
    _getword: c_int,
    _flags: c_int,
    _winp: *mut *mut c_void,
    _lnump: *mut c_long,
    _textp: *mut *mut c_uchar,
    _colp: *mut c_int,
    _startcolp: *mut c_int,
) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn get_beval_info(
    _beval: *mut c_void,
    _getword: c_int,
    _winp: *mut *mut c_void,
    _lnump: *mut c_long,
    _textp: *mut *mut c_uchar,
    _colp: *mut c_int,
) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn post_balloon(
    _beval: *mut c_void,
    _mesg: *mut c_uchar,
    _list: *mut c_void,
) {
    // no-op
}

#[no_mangle]
pub extern "C" fn can_use_beval() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn general_beval_cb(_beval: *mut c_void, _state: c_int) {
    // no-op
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beval_stubs_return_defaults() {
        assert_eq!(can_use_beval(), 0);
        assert_eq!(get_beval_info(std::ptr::null_mut(), 0, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()), 0);
    }
}
