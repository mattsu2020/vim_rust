use libc::{c_char, c_uchar, c_void};
use std::ptr;

#[no_mangle]
pub unsafe extern "C" fn vim_strsave(string: *const c_uchar) -> *mut c_uchar {
    if string.is_null() {
        return ptr::null_mut();
    }
    let len = libc::strlen(string as *const c_char) + 1;
    let p = libc::malloc(len) as *mut c_uchar;
    if p.is_null() {
        return ptr::null_mut();
    }
    libc::memcpy(p as *mut c_void, string as *const c_void, len);
    p
}

#[no_mangle]
pub unsafe extern "C" fn vim_strnsave(string: *const c_uchar, len: usize) -> *mut c_uchar {
    let p = libc::malloc(len + 1) as *mut c_uchar;
    if p.is_null() {
        return ptr::null_mut();
    }
    if !string.is_null() {
        libc::memcpy(p as *mut c_void, string as *const c_void, len);
    }
    *p.add(len) = 0;
    p
}
