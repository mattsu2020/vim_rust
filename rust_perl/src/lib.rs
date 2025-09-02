use perl_sys::*;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// Execute a snippet of Perl code.
#[no_mangle]
pub extern "C" fn vim_perl_exec(code: *const c_char) -> c_int {
    if code.is_null() {
        return 0;
    }
    unsafe {
        let c_str = CStr::from_ptr(code);
        eval_pv(c_str.to_str().unwrap_or(""), 0);
    }
    1
}
