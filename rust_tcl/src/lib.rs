use std::os::raw::{c_char, c_int};
use tcl_sys::{Tcl_CreateInterp, Tcl_DeleteInterp, Tcl_Eval, TCL_OK};

fn run_code(code: *const c_char) -> Result<(), ()> {
    if code.is_null() {
        return Err(());
    }
    unsafe {
        let interp = Tcl_CreateInterp();
        if interp.is_null() {
            return Err(());
        }
        let result = Tcl_Eval(interp, code);
        Tcl_DeleteInterp(interp);
        if result == TCL_OK as i32 {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// Execute a Tcl command string.
#[no_mangle]
pub extern "C" fn vim_tcl_exec(code: *const c_char) -> c_int {
    run_code(code).map_or(0, |_| 1)
}

/// Alias exposed for compatibility with existing Vim commands.
#[no_mangle]
pub extern "C" fn tcl_execute(code: *const c_char) -> c_int {
    vim_tcl_exec(code)
}
