use std::os::raw::c_int;

#[repr(C)]
pub struct oparg_T {
    _private: [u8; 0],
}

#[cfg(not(test))]
extern "C" {
    fn normal_cmd_c(oap: *mut oparg_T, toplevel: c_int);
}

#[cfg(test)]
#[no_mangle]
extern "C" fn normal_cmd_c(_oap: *mut oparg_T, _toplevel: c_int) {}

#[no_mangle]
pub extern "C" fn rs_normal_cmd(oap: *mut oparg_T, toplevel: c_int) {
    unsafe {
        normal_cmd_c(oap, toplevel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        unsafe { rs_normal_cmd(std::ptr::null_mut(), 0) };
    }
}
