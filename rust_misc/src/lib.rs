use std::os::raw::{c_int, c_uint, c_uchar};

// Constants matching the original C implementation.
const URL_SLASH: c_int = 1;
const URL_BACKSLASH: c_int = 2;
const CTRL_V: c_int = 22; // Ctrl-V character
const MODE_INSERT: c_int = 0x10;
const VE_BLOCK: c_uint = 5;
const VE_INSERT: c_uint = 6;
const VE_ALL: c_uint = 4;
const MAYBE: c_int = 2;

#[no_mangle]
pub static mut virtual_op: c_int = MAYBE;
#[no_mangle]
pub static mut VIsual_active: c_int = 0;
#[no_mangle]
pub static mut VIsual_mode: c_int = 0;
#[no_mangle]
pub static mut State: c_int = 0;
#[no_mangle]
pub static mut VE_FLAGS: c_uint = 0;

#[no_mangle]
pub extern "C" fn get_ve_flags() -> c_uint {
    unsafe { VE_FLAGS & !0 } // simple placeholder, masks not needed
}

#[no_mangle]
pub unsafe extern "C" fn path_is_url(p: *const c_uchar) -> c_int {
    if p.is_null() {
        return 0;
    }
    if *p.add(0) == b':' && *p.add(1) == b'/' && *p.add(2) == b'/' {
        URL_SLASH
    } else if *p.add(0) == b':' && *p.add(1) == b'\\' && *p.add(2) == b'\\' {
        URL_BACKSLASH
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn path_with_url(fname: *const c_uchar) -> c_int {
    if fname.is_null() {
        return 0;
    }
    let mut p = fname;
    let first = *p as char;
    if !first.is_ascii_alphabetic() {
        return 0;
    }
    p = p.add(1);
    while {
        let ch = *p as char;
        ch.is_ascii_alphabetic() || ch == '-'
    } {
        p = p.add(1);
    }
    if *p.offset(-1) == b'-' {
        return 0;
    }
    path_is_url(p)
}

#[no_mangle]
pub unsafe extern "C" fn virtual_active() -> c_int {
    let cur_ve_flags = get_ve_flags();
    if virtual_op != MAYBE {
        return virtual_op;
    }
    if cur_ve_flags == VE_ALL
        || (cur_ve_flags & VE_BLOCK != 0 && VIsual_active != 0 && VIsual_mode == CTRL_V)
        || (cur_ve_flags & VE_INSERT != 0 && (State & MODE_INSERT) != 0)
    {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_path_is_url() {
        let s = b"://example";
        assert_eq!(unsafe { path_is_url(s.as_ptr()) }, URL_SLASH);
        let s = b":\\\\foo";
        assert_eq!(unsafe { path_is_url(s.as_ptr()) }, URL_BACKSLASH);
        let s = b"not";
        assert_eq!(unsafe { path_is_url(s.as_ptr()) }, 0);
    }

    #[test]
    fn test_path_with_url() {
        let s = b"http://vim.org";
        assert_eq!(unsafe { path_with_url(s.as_ptr()) }, URL_SLASH);
        let s = b"ftp\\\\server";
        assert_eq!(unsafe { path_with_url(s.as_ptr()) }, URL_BACKSLASH);
        let s = b"abc";
        assert_eq!(unsafe { path_with_url(s.as_ptr()) }, 0);
    }
}
