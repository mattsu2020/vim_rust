use std::os::raw::{c_char, c_int};
use tcl_sys::{Tcl_CreateInterp, Tcl_DeleteInterp, Tcl_Eval, TCL_OK};

use rust_ffi::{cstr_to_str, result_to_int, FFIError, FFIResult};

fn run_code(code: *const c_char) -> FFIResult<()> {
    cstr_to_str(code)?;
    unsafe {
        let interp = Tcl_CreateInterp();
        if interp.is_null() {
            return Err(FFIError::Exec);
        }
        let result = Tcl_Eval(interp, code);
        Tcl_DeleteInterp(interp);
        if result == TCL_OK as i32 {
            Ok(())
        } else {
            Err(FFIError::Exec)
        }
    }
}

/// Execute a Tcl command string.
#[no_mangle]
pub extern "C" fn vim_tcl_exec(code: *const c_char) -> c_int {
    result_to_int(run_code(code))
}

/// Alias exposed for compatibility with existing Vim commands.
#[no_mangle]
pub extern "C" fn tcl_execute(code: *const c_char) -> c_int {
    vim_tcl_exec(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn exec_and_null_error() {
        let code = CString::new("set a 1").unwrap();
        assert_eq!(vim_tcl_exec(code.as_ptr()), 1);
        assert_eq!(vim_tcl_exec(std::ptr::null()), 0);
    }
}
