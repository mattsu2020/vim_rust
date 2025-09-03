use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct oparg_T {
    _private: [u8; 0],
}

#[cfg(not(test))]
extern "C" {
    fn normal_cmd_c(oap: *mut oparg_T, toplevel: c_int);
}

#[cfg(test)]
#[no_mangle]
extern "C" fn normal_cmd_c(_oap: *mut oparg_T, _toplevel: c_int) {}

#[no_mangle]
pub extern "C" fn rs_normal_cmd(oap: *mut oparg_T, toplevel: c_int) {
    unsafe {
        normal_cmd_c(oap, toplevel);
    }
}

const SHOWCMD_BUFLEN: usize = 41;

#[cfg(not(test))]
extern "C" {
    static mut p_sc: c_int;
    static mut showcmd_buf: [c_char; SHOWCMD_BUFLEN];
    fn char_avail() -> c_int;
    fn display_showcmd();
}

#[cfg(test)]
#[no_mangle]
static mut p_sc: c_int = 0;
#[cfg(test)]
#[no_mangle]
static mut showcmd_buf: [c_char; SHOWCMD_BUFLEN] = [0; SHOWCMD_BUFLEN];
#[cfg(test)]
#[no_mangle]
extern "C" fn char_avail() -> c_int { 0 }
#[cfg(test)]
#[no_mangle]
extern "C" fn display_showcmd() {}

#[no_mangle]
pub extern "C" fn rs_del_from_showcmd(len: c_int) {
    unsafe {
        if p_sc == 0 {
            return;
        }

        let mut len = len;
        let old_len =
            CStr::from_ptr(showcmd_buf.as_ptr()).to_bytes().len() as c_int;
        if len > old_len {
            len = old_len;
        }
        *showcmd_buf
            .as_mut_ptr()
            .add((old_len - len) as usize) = 0;

        if char_avail() == 0 {
            display_showcmd();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn smoke_test() {
        unsafe { rs_normal_cmd(std::ptr::null_mut(), 0) };
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
            let res = CStr::from_ptr(showcmd_buf.as_ptr())
                .to_str()
                .unwrap();
            assert_eq!(res, "ab");
        }
    }
}
