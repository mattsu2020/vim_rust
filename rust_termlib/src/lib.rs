use std::os::raw::{c_char, c_int};
use std::ptr;

// Exported globals matching the original C implementation.
#[no_mangle]
pub static mut PC: c_char = 0;
#[no_mangle]
pub static mut UP: *const c_char = ptr::null();
#[no_mangle]
pub static mut BC: *const c_char = ptr::null();
#[no_mangle]
pub static mut ospeed: i16 = 0;

// Simple placeholders -------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn tgetent(tbuf: *mut c_char, _term: *const c_char) -> c_int {
    if tbuf.is_null() {
        return -1;
    }
    // Ensure buffer is NUL terminated but otherwise leave empty.
    *tbuf = 0;
    1
}

#[no_mangle]
pub unsafe extern "C" fn tgetflag(_id: *const c_char) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn tgetnum(_id: *const c_char) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn tgetstr(_id: *const c_char, _buf: *mut *mut c_char) -> *mut c_char {
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn tgoto(cm: *mut c_char, _col: c_int, _line: c_int) -> *mut c_char {
    cm
}

#[no_mangle]
pub unsafe extern "C" fn tputs(
    cp: *const c_char,
    _affcnt: c_int,
    outc: Option<extern "C" fn(u32)>,
) -> c_int {
    if cp.is_null() {
        return 0;
    }
    let mut p = cp;
    while *p != 0 {
        if let Some(func) = outc {
            func(*p as u32);
        }
        p = p.add(1);
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    extern "C" fn collect(c: u32) {
        assert!(c < 256);
    }

    #[test]
    fn basic_calls() {
        let buf = vec![0u8; 1];
        unsafe {
            assert_eq!(tgetent(buf.as_ptr() as *mut c_char, ptr::null()), 1);
            let s = CString::new("hi").unwrap();
            tputs(s.as_ptr(), 1, Some(collect));
        }
    }
}
