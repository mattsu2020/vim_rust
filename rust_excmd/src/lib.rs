use std::os::raw::{c_int, c_void};
use std::ptr;

pub type CharU = u8;
pub type GetlineOpt = c_int;

pub type Fgetline = Option<unsafe extern "C" fn(c_int, *mut c_void, c_int, GetlineOpt) -> *mut CharU>;

const FAIL: c_int = 0;
const OK: c_int = 1;
const MAGIC_ON: c_int = 3;
const MAGIC_ALL: c_int = 4;

extern "C" {
    static mut trylevel: c_int;
    static mut emsg_silent: c_int;
    static mut force_abort: c_int;
    fn aborting() -> c_int;
}

// used for get_pressedreturn()/set_pressedreturn()
static mut EX_PRESSEDRETURN: c_int = 0;

#[no_mangle]
pub extern "C" fn do_cmdline(
    mut cmdline: *mut CharU,
    fgetline: Fgetline,
    cookie: *mut c_void,
    flags: c_int,
) -> c_int {
    unsafe {
        loop {
            if cmdline.is_null() || *cmdline == 0 {
                if let Some(fg) = fgetline {
                    cmdline = fg(0, cookie, 0, 0);
                    if cmdline.is_null() {
                        break;
                    }
                } else {
                    break;
                }
            }
            let mut cmdp = cmdline;
            let next = do_one_cmd(&mut cmdp, flags, ptr::null_mut(), fgetline, cookie);
            if next.is_null() {
                break;
            }
            cmdline = next;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn do_one_cmd(
    cmdlinep: *mut *mut CharU,
    _flags: c_int,
    _cstack: *mut c_void,
    _fgetline: Fgetline,
    _cookie: *mut c_void,
) -> *mut CharU {
    unsafe {
        let cmdline = *cmdlinep;
        if cmdline.is_null() || *cmdline == 0 {
            return ptr::null_mut();
        }
        let mut p = cmdline;
        while *p != 0 && *p != b'|' && *p != b'\n' {
            p = p.add(1);
        }
        if *p == 0 {
            *cmdlinep = p;
            ptr::null_mut()
        } else {
            *p = 0;
            p = p.add(1);
            *cmdlinep = p;
            p
        }
    }
}

#[no_mangle]
pub extern "C" fn add_bufnum(bufnrs: *mut c_int, bufnump: *mut c_int, nr: c_int) {
    unsafe {
        let mut i = 0;
        while i < *bufnump {
            if *bufnrs.add(i as usize) == nr {
                return;
            }
            i += 1;
        }
        *bufnrs.add(*bufnump as usize) = nr;
        *bufnump += 1;
    }
}

#[no_mangle]
pub extern "C" fn rust_empty_pattern_magic(p: *const CharU, len: usize, magic_val: c_int) -> c_int {
    unsafe {
        let mut len = len as isize;
        while len >= 2
            && *p.offset(len - 2) == b'\\'
            && b"mMvVcCZ".contains(&*p.offset(len - 1))
        {
            len -= 2;
        }
        if len == 0 {
            1
        } else if len > 1
            && *p.offset(len - 1) == b'|'
            && ((*p.offset(len - 2) == b'\\' && magic_val == MAGIC_ON)
                || (*p.offset(len - 2) != b'\\' && magic_val == MAGIC_ALL))
        {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_should_abort(retcode: c_int) -> c_int {
    unsafe {
        if (retcode == FAIL && trylevel != 0 && emsg_silent == 0) || aborting() != 0 {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_update_force_abort(cause_abort: c_int) {
    unsafe {
        if cause_abort != 0 {
            force_abort = 1;
        }
    }
}

// ----- functions originally from ex_cmds2.c -----

#[no_mangle]
pub extern "C" fn autowrite(_buf: *mut c_void, _forceit: c_int) -> c_int {
    // no-op stub, always fail
    FAIL
}

#[no_mangle]
pub extern "C" fn autowrite_all() {}

#[no_mangle]
pub extern "C" fn check_changed(_buf: *mut c_void, _flags: c_int) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn browse_save_fname(_buf: *mut c_void) {}

#[no_mangle]
pub extern "C" fn dialog_changed(_buf: *mut c_void, _checkall: c_int) {}

#[no_mangle]
pub extern "C" fn can_abandon(_buf: *mut c_void, _forceit: c_int) -> c_int {
    1
}

#[no_mangle]
pub extern "C" fn check_changed_any(_hidden: c_int, _unload: c_int) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn check_fname() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn buf_write_all(_buf: *mut c_void, _forceit: c_int) -> c_int {
    OK
}

#[no_mangle]
pub extern "C" fn ex_listdo(_eap: *mut c_void) {}

#[no_mangle]
pub extern "C" fn ex_compiler(_eap: *mut c_void) {}

#[no_mangle]
pub extern "C" fn init_pyxversion() {}

#[no_mangle]
pub extern "C" fn ex_pyxfile(_eap: *mut c_void) {}

#[no_mangle]
pub extern "C" fn ex_pyx(_eap: *mut c_void) {}

#[no_mangle]
pub extern "C" fn ex_pyxdo(_eap: *mut c_void) {}

#[no_mangle]
pub extern "C" fn ex_checktime(_eap: *mut c_void) {}

// ----- replacements for get_pressedreturn/set_pressedreturn -----

#[no_mangle]
pub extern "C" fn get_pressedreturn() -> c_int {
    unsafe { EX_PRESSEDRETURN }
}

#[no_mangle]
pub extern "C" fn set_pressedreturn(val: c_int) {
    unsafe { EX_PRESSEDRETURN = val; }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[no_mangle]
    static mut trylevel: c_int = 0;
    #[no_mangle]
    static mut emsg_silent: c_int = 0;
    #[no_mangle]
    static mut force_abort: c_int = 0;

    #[no_mangle]
    extern "C" fn aborting() -> c_int { 0 }

    #[test]
    fn call_stubs() {
        let mut buf = b"cmd1|cmd2".to_vec();
        buf.push(0);
        let mut cmdline = buf.as_mut_ptr();
        let res = do_cmdline(cmdline, None, ptr::null_mut(), 0);
        assert_eq!(res, 0);

        let mut buf2 = b"abc|def".to_vec();
        buf2.push(0);
        let mut ptrp = buf2.as_mut_ptr();
        let next = do_one_cmd(&mut ptrp, 0, ptr::null_mut(), None, ptr::null_mut());
        assert!(!next.is_null());

        let mut bufs = [1, 2, 0];
        let mut num = 2;
        add_bufnum(bufs.as_mut_ptr(), &mut num, 3);
        assert_eq!(num, 3);

        assert_eq!(get_pressedreturn(), 0);
        set_pressedreturn(1);
        assert_eq!(get_pressedreturn(), 1);

        let pat = b"foo\\|bar";
        let res = rust_empty_pattern_magic(pat.as_ptr(), pat.len(), MAGIC_ON);
        assert_eq!(res, 0);

        assert_eq!(rust_should_abort(0), 0);
        unsafe { force_abort = 0; }
        rust_update_force_abort(1);
        unsafe { assert_eq!(force_abort, 1); }
    }
}
