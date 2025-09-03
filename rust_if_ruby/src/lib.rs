use magnus::{eval, Value};
use std::os::raw::{c_char, c_int};

use rust_ffi::{cstr_to_str, result_to_int, FFIError, FFIResult};

fn run_code(code: *const c_char) -> FFIResult<()> {
    let source = cstr_to_str(code)?;
    let _cleanup = unsafe { magnus::embed::init() };
    eval::<Value>(source).map(|_| ()).map_err(|_| FFIError::Exec)
}

/// Execute Ruby code using the embedded interpreter.
#[no_mangle]
pub extern "C" fn vim_ruby_exec(code: *const c_char) -> c_int {
    result_to_int(run_code(code))
}

/// Alias exposed for compatibility with existing Vim commands.
#[no_mangle]
pub extern "C" fn ruby_execute(code: *const c_char) -> c_int {
    vim_ruby_exec(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn exec_and_null_error() {
        let code = CString::new("1 + 1").unwrap();
        assert_eq!(vim_ruby_exec(code.as_ptr()), 1);
        assert_eq!(vim_ruby_exec(std::ptr::null()), 0);
    }
}
