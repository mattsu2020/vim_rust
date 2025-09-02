use magnus::{eval, Value};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

fn run_code(code: *const c_char) -> Result<(), ()> {
    if code.is_null() {
        return Err(());
    }
    let source = unsafe { CStr::from_ptr(code) }.to_str().map_err(|_| ())?;
    let _cleanup = unsafe { magnus::embed::init() };
    eval::<Value>(source).map(|_| ()).map_err(|_| ())
}

/// Execute Ruby code using the embedded interpreter.
#[no_mangle]
pub extern "C" fn vim_ruby_exec(code: *const c_char) -> c_int {
    run_code(code).map_or(0, |_| 1)
}

/// Alias exposed for compatibility with existing Vim commands.
#[no_mangle]
pub extern "C" fn ruby_execute(code: *const c_char) -> c_int {
    vim_ruby_exec(code)
}
