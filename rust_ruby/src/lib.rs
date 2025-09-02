use magnus::{eval, Value};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// Execute Ruby code using the embedded interpreter.
#[no_mangle]
pub extern "C" fn vim_ruby_exec(code: *const c_char) -> c_int {
    if code.is_null() {
        return 0;
    }
    let c_str = unsafe { CStr::from_ptr(code) };
    let source = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    if magnus::embed::init().is_err() {
        return 0;
    }
    match eval::<Value>(source) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}
