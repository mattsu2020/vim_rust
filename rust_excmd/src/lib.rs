use std::os::raw::{c_int, c_void};

pub type CharU = u8;
pub type GetlineOpt = c_int;

pub type Fgetline = Option<unsafe extern "C" fn(c_int, *mut c_void, c_int, GetlineOpt) -> *mut CharU>;

#[no_mangle]
pub extern "C" fn rust_do_cmdline(
    _cmdline: *mut CharU,
    _fgetline: Fgetline,
    _cookie: *mut c_void,
    _flags: c_int,
) -> c_int {
    // Placeholder implementation delegating Ex command processing to Rust.
    0
}

#[no_mangle]
pub extern "C" fn rust_do_one_cmd(
    _cmdlinep: *mut *mut CharU,
    _flags: c_int,
    _cstack: *mut c_void,
    _fgetline: Fgetline,
    _cookie: *mut c_void,
) -> *mut CharU {
    // Placeholder that currently does nothing and returns NULL.
    std::ptr::null_mut()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_stubs() {
        let mut cmd: *mut CharU = std::ptr::null_mut();
        let res = rust_do_cmdline(cmd, None, std::ptr::null_mut(), 0);
        assert_eq!(res, 0);

        let res2 = rust_do_one_cmd(&mut cmd, 0, std::ptr::null_mut(), None, std::ptr::null_mut());
        assert!(res2.is_null());
    }
}
