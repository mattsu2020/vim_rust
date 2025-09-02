use std::os::raw::{c_char, c_int};
use tcl_sys::{Tcl_CreateInterp, Tcl_DeleteInterp, Tcl_Eval, TCL_OK};

/// Execute a Tcl command string.
#[no_mangle]
pub extern "C" fn vim_tcl_exec(code: *const c_char) -> c_int {
    if code.is_null() {
        return 0;
    }
    unsafe {
        let interp = Tcl_CreateInterp();
        if interp.is_null() {
            return 0;
        }
        let result = Tcl_Eval(interp, code);
        Tcl_DeleteInterp(interp);
        if result == TCL_OK as i32 { 1 } else { 0 }
    }
}
