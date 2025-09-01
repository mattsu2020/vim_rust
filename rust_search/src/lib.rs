use libc::{c_int, c_long, size_t, c_uchar};
use std::ffi::CStr;

#[repr(C)]
pub struct win_T { _private: [u8; 0] }
#[repr(C)]
pub struct buf_T { _private: [u8; 0] }
#[repr(C)]
pub struct searchit_arg_T { _private: [u8; 0] }

#[repr(C)]
pub struct pos_T {
    pub lnum: c_long,
    pub col: c_int,
    pub coladd: c_int,
}

#[no_mangle]
pub extern "C" fn rust_searchit(
    _win: *mut win_T,
    _buf: *mut buf_T,
    pos: *mut pos_T,
    _end_pos: *mut pos_T,
    _dir: c_int,
    pat: *const c_uchar,
    patlen: size_t,
    _count: c_long,
    _options: c_int,
    _pat_use: c_int,
    _extra_arg: *mut searchit_arg_T,
) -> c_int {
    if pos.is_null() || pat.is_null() || patlen == 0 {
        return 0;
    }
    // For now, simply set position to start and report success.
    unsafe {
        (*pos).col = 0;
    }
    1
}

#[no_mangle]
pub extern "C" fn rust_find_pattern_in_path(
    ptr: *mut c_uchar,
    _dir: c_int,
    _len: c_int,
    _whole: c_int,
    _skip_comments: c_int,
    _typ: c_int,
    _count: c_long,
    _action: c_int,
    _start_lnum: c_long,
    _end_lnum: c_long,
    _forceit: c_int,
    _silent: c_int,
) {
    if ptr.is_null() {
        return;
    }
    // Simple stub: print pattern to stderr for debugging.
    let pat = unsafe { CStr::from_ptr(ptr as *const i8).to_string_lossy() };
    eprintln!("rust_find_pattern_in_path: {}", pat);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn searchit_sets_col() {
        let mut pos = pos_T { lnum: 1, col: 5, coladd: 0 };
        let pat = b"pat\0";
        let r = unsafe {
            rust_searchit(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut pos,
                std::ptr::null_mut(),
                1,
                pat.as_ptr(),
                3,
                1,
                0,
                0,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(r, 1);
        assert_eq!(pos.col, 0);
    }
}
