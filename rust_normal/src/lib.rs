use std::os::raw::c_int;

#[repr(C)]
pub struct oparg_T {
    _private: [u8; 0],
}

#[cfg(not(test))]
extern "C" {
    fn normal_cmd_c(oap: *mut oparg_T, toplevel: c_int);
    fn op_change_c(oap: *mut oparg_T) -> c_int;
}

#[cfg(test)]
#[no_mangle]
extern "C" fn normal_cmd_c(_oap: *mut oparg_T, _toplevel: c_int) {}

#[cfg(test)]
#[no_mangle]
extern "C" fn op_change_c(_oap: *mut oparg_T) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn rs_normal_cmd(oap: *mut oparg_T, toplevel: c_int) {
    unsafe {
        normal_cmd_c(oap, toplevel);
    }
}

#[no_mangle]
pub extern "C" fn rs_op_change(oap: *mut oparg_T) -> c_int {
    unsafe { op_change_c(oap) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        let _ = unsafe { rs_op_change(std::ptr::null_mut()) };
    }
}
