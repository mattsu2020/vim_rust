use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct oparg_T {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn rs_normal_cmd(_oap: *mut oparg_T, _toplevel: c_int) {
    // Normal mode command execution is now implemented in Rust. The current
    // stub keeps the interface compatible while the full port is in progress.
}

const SHOWCMD_BUFLEN: usize = 41;
const SHOWCMD_COLS: usize = 10;
const MB_MAXBYTES: usize = 21;

#[cfg(not(test))]
extern "C" {
    static mut p_sc: c_int;
    static mut msg_silent: c_int;
    static mut showcmd_buf: [c_char; SHOWCMD_BUFLEN];
    static mut showcmd_visual: c_int;
    fn char_avail() -> c_int;
    fn display_showcmd();
    fn vim_isprintc(c: c_int) -> c_int;
    fn transchar(c: c_int) -> *const c_char;
    fn mb_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn setcursor();
    fn text_locked() -> c_int;
    fn text_locked_msg();
    fn curbuf_locked() -> c_int;
    fn clearop(oap: *mut oparg_T);
    fn clearopbeep(oap: *mut oparg_T);
}

#[cfg(test)]
#[no_mangle]
static mut p_sc: c_int = 0;
#[cfg(test)]
#[no_mangle]
static mut msg_silent: c_int = 0;
#[cfg(test)]
#[no_mangle]
static mut showcmd_buf: [c_char; SHOWCMD_BUFLEN] = [0; SHOWCMD_BUFLEN];
#[cfg(test)]
#[no_mangle]
static mut showcmd_visual: c_int = 0;
#[cfg(test)]
#[no_mangle]
extern "C" fn char_avail() -> c_int {
    0
}
#[cfg(test)]
#[no_mangle]
extern "C" fn display_showcmd() {}
#[cfg(test)]
#[no_mangle]
extern "C" fn vim_isprintc(_c: c_int) -> c_int {
    1
}
#[cfg(test)]
static mut TRANSCHAR_BUF: [c_char; 8] = [0; 8];
#[cfg(test)]
#[no_mangle]
extern "C" fn transchar(c: c_int) -> *const c_char {
    unsafe {
        TRANSCHAR_BUF[0] = c as c_char;
        TRANSCHAR_BUF[1] = 0;
        TRANSCHAR_BUF.as_ptr()
    }
}
#[cfg(test)]
#[no_mangle]
extern "C" fn mb_char2bytes(c: c_int, buf: *mut c_char) -> c_int {
    if c > 0x7f {
        0
    } else {
        unsafe {
            *buf = c as c_char;
        }
        1
    }
}
#[cfg(test)]
#[no_mangle]
extern "C" fn setcursor() {}

#[cfg(test)]
#[no_mangle]
static mut TEXT_LOCKED_RET: c_int = 0;
#[cfg(test)]
#[no_mangle]
static mut CURBUF_LOCKED_RET: c_int = 0;
#[cfg(test)]
#[no_mangle]
static mut CLEAROP_CALLED: c_int = 0;
#[cfg(test)]
#[no_mangle]
static mut CLEAROPBEEP_CALLED: c_int = 0;

#[cfg(test)]
#[no_mangle]
extern "C" fn text_locked() -> c_int {
    unsafe { TEXT_LOCKED_RET }
}
#[cfg(test)]
#[no_mangle]
extern "C" fn text_locked_msg() {}
#[cfg(test)]
#[no_mangle]
extern "C" fn curbuf_locked() -> c_int {
    unsafe { CURBUF_LOCKED_RET }
}
#[cfg(test)]
#[no_mangle]
extern "C" fn clearop(_oap: *mut oparg_T) {
    unsafe {
        CLEAROP_CALLED += 1;
    }
}
#[cfg(test)]
#[no_mangle]
extern "C" fn clearopbeep(_oap: *mut oparg_T) {
    unsafe {
        CLEAROPBEEP_CALLED += 1;
    }
}

#[no_mangle]
pub extern "C" fn rs_del_from_showcmd(len: c_int) {
    unsafe {
        if p_sc == 0 {
            return;
        }

        let mut len = len;
        let old_len = CStr::from_ptr(showcmd_buf.as_ptr()).to_bytes().len() as c_int;
        if len > old_len {
            len = old_len;
        }
        *showcmd_buf.as_mut_ptr().add((old_len - len) as usize) = 0;

        if char_avail() == 0 {
            display_showcmd();
        }
    }
}

unsafe fn add_to_showcmd_inner(c: c_int) -> Option<bool> {
    if p_sc == 0 || msg_silent != 0 {
        return Some(false);
    }
    if showcmd_visual != 0 {
        showcmd_buf[0] = 0;
        showcmd_visual = 0;
    }
    if c < 0 {
        return Some(false);
    }
    let add_bytes: &[u8] = if c <= 0x7f || vim_isprintc(c) == 0 {
        let p = transchar(c);
        if p.is_null() {
            return None;
        }
        let bytes = CStr::from_ptr(p).to_bytes();
        if bytes == b" " {
            b"<20>"
        } else {
            bytes
        }
    } else {
        let mut buf = [0 as c_char; MB_MAXBYTES + 1];
        let len = mb_char2bytes(c, buf.as_mut_ptr());
        if len <= 0 {
            return None;
        }
        buf[len as usize] = 0;
        CStr::from_ptr(buf.as_ptr()).to_bytes()
    };

    let old_len = CStr::from_ptr(showcmd_buf.as_ptr()).to_bytes().len();
    let extra_len = add_bytes.len();
    let overflow = old_len + extra_len;
    let mut start = old_len;
    if overflow > SHOWCMD_COLS {
        let shift = overflow - SHOWCMD_COLS;
        if shift > old_len {
            showcmd_buf[0] = 0;
            start = 0;
        } else {
            for i in 0..=old_len - shift {
                showcmd_buf[i] = showcmd_buf[i + shift];
            }
            start = old_len - shift;
        }
    }
    for (i, &b) in add_bytes.iter().enumerate() {
        showcmd_buf[start + i] = b as c_char;
    }
    showcmd_buf[start + extra_len] = 0;

    if char_avail() != 0 {
        return Some(false);
    }
    display_showcmd();
    Some(true)
}

#[no_mangle]
pub extern "C" fn rs_add_to_showcmd(c: c_int) -> c_int {
    match unsafe { add_to_showcmd_inner(c) } {
        Some(true) => 1,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn rs_add_to_showcmd_c(c: c_int) {
    unsafe {
        if add_to_showcmd_inner(c) != Some(true) {
            setcursor();
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_check_text_or_curbuf_locked(oap: *mut oparg_T) -> c_int {
    unsafe {
        if text_locked() != 0 {
            if !oap.is_null() {
                clearopbeep(oap);
            }
            text_locked_msg();
            return 1;
        }
        if curbuf_locked() == 0 {
            return 0;
        }
        if !oap.is_null() {
            clearop(oap);
        }
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn smoke_test() {
        rs_normal_cmd(std::ptr::null_mut(), 0);
    }

    #[test]
    fn del_from_showcmd_basic() {
        unsafe {
            p_sc = 1;
            let initial = b"abcd\0";
            ptr::copy_nonoverlapping(
                initial.as_ptr() as *const c_char,
                showcmd_buf.as_mut_ptr(),
                initial.len(),
            );
            rs_del_from_showcmd(2);
            let res = CStr::from_ptr(showcmd_buf.as_ptr()).to_str().unwrap();
            assert_eq!(res, "ab");
        }
    }

    #[test]
    fn add_to_showcmd_basic() {
        unsafe {
            p_sc = 1;
            msg_silent = 0;
            showcmd_visual = 0;
            let initial = b"ab\0";
            ptr::copy_nonoverlapping(
                initial.as_ptr() as *const c_char,
                showcmd_buf.as_mut_ptr(),
                initial.len(),
            );
            let r = rs_add_to_showcmd('c' as c_int);
            assert_eq!(r, 1);
            let res = CStr::from_ptr(showcmd_buf.as_ptr()).to_str().unwrap();
            assert_eq!(res, "abc");
        }
    }

    #[test]
    fn add_to_showcmd_invalid_keeps_buffer() {
        unsafe {
            p_sc = 1;
            msg_silent = 0;
            showcmd_visual = 0;
            let initial = b"ab\0";
            ptr::copy_nonoverlapping(
                initial.as_ptr() as *const c_char,
                showcmd_buf.as_mut_ptr(),
                initial.len(),
            );
            let r = rs_add_to_showcmd(0x110000);
            assert_eq!(r, 0);
            let res = CStr::from_ptr(showcmd_buf.as_ptr()).to_str().unwrap();
            assert_eq!(res, "ab");
        }
    }

    #[test]
    fn check_text_locked_calls_beep() {
        unsafe {
            TEXT_LOCKED_RET = 1;
            CURBUF_LOCKED_RET = 0;
            CLEAROPBEEP_CALLED = 0;
            let oap = std::ptr::NonNull::<oparg_T>::dangling().as_ptr();
            let r = rs_check_text_or_curbuf_locked(oap);
            assert_eq!(r, 1);
            assert_eq!(CLEAROPBEEP_CALLED, 1);
        }
    }

    #[test]
    fn check_curbuf_locked_calls_clearop() {
        unsafe {
            TEXT_LOCKED_RET = 0;
            CURBUF_LOCKED_RET = 1;
            CLEAROP_CALLED = 0;
            let oap = std::ptr::NonNull::<oparg_T>::dangling().as_ptr();
            let r = rs_check_text_or_curbuf_locked(oap);
            assert_eq!(r, 1);
            assert_eq!(CLEAROP_CALLED, 1);
        }
    }

    #[test]
    fn check_not_locked_returns_zero() {
        unsafe {
            TEXT_LOCKED_RET = 0;
            CURBUF_LOCKED_RET = 0;
            let r = rs_check_text_or_curbuf_locked(std::ptr::null_mut());
            assert_eq!(r, 0);
        }
    }
}
