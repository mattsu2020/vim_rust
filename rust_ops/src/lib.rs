#![allow(unused_unsafe, non_snake_case)]

use std::os::raw::{c_int, c_long};

#[repr(C)]
pub struct oparg_T {
    _private: [u8; 0],
}

#[cfg(not(test))]
extern "C" {
    fn op_shift_c(oap: *mut oparg_T, curs_top: c_int, amount: c_int);
    fn op_delete_c(oap: *mut oparg_T) -> c_int;
    fn op_replace_c(oap: *mut oparg_T, c: c_int) -> c_int;
    fn op_tilde_c(oap: *mut oparg_T);
    fn op_insert_c(oap: *mut oparg_T, count1: c_long);
    fn op_change_c(oap: *mut oparg_T) -> c_int;
    fn op_addsub_c(oap: *mut oparg_T, Prenum1: c_long, g_cmd: c_int);
    fn op_colon_c(oap: *mut oparg_T);
    fn op_function_c(oap: *mut oparg_T);
}

#[cfg(test)]
#[no_mangle]
extern "C" fn op_shift_c(_oap: *mut oparg_T, _curs_top: c_int, _amount: c_int) {}
#[cfg(test)]
#[no_mangle]
extern "C" fn op_delete_c(_oap: *mut oparg_T) -> c_int { 0 }
#[cfg(test)]
#[no_mangle]
extern "C" fn op_replace_c(_oap: *mut oparg_T, _c: c_int) -> c_int { 0 }
#[cfg(test)]
#[no_mangle]
extern "C" fn op_tilde_c(_oap: *mut oparg_T) {}
#[cfg(test)]
#[no_mangle]
extern "C" fn op_insert_c(_oap: *mut oparg_T, _count1: c_long) {}
#[cfg(test)]
#[no_mangle]
extern "C" fn op_change_c(_oap: *mut oparg_T) -> c_int { 0 }
#[cfg(test)]
#[no_mangle]
extern "C" fn op_addsub_c(_oap: *mut oparg_T, _Prenum1: c_long, _g_cmd: c_int) {}
#[cfg(test)]
#[no_mangle]
extern "C" fn op_colon_c(_oap: *mut oparg_T) {}
#[cfg(test)]
#[no_mangle]
extern "C" fn op_function_c(_oap: *mut oparg_T) {}

#[no_mangle]
pub extern "C" fn rs_op_shift(oap: *mut oparg_T, curs_top: c_int, amount: c_int) {
    unsafe { op_shift_c(oap, curs_top, amount) }
}

#[no_mangle]
pub extern "C" fn rs_op_delete(oap: *mut oparg_T) -> c_int {
    unsafe { op_delete_c(oap) }
}

#[no_mangle]
pub extern "C" fn rs_op_replace(oap: *mut oparg_T, c: c_int) -> c_int {
    unsafe { op_replace_c(oap, c) }
}

#[no_mangle]
pub extern "C" fn rs_op_tilde(oap: *mut oparg_T) {
    unsafe { op_tilde_c(oap) }
}

#[no_mangle]
pub extern "C" fn rs_op_insert(oap: *mut oparg_T, count1: c_long) {
    unsafe { op_insert_c(oap, count1) }
}

#[no_mangle]
pub extern "C" fn rs_op_change(oap: *mut oparg_T) -> c_int {
    unsafe { op_change_c(oap) }
}

#[no_mangle]
pub extern "C" fn rs_op_addsub(oap: *mut oparg_T, Prenum1: c_long, g_cmd: c_int) {
    unsafe { op_addsub_c(oap, Prenum1, g_cmd) }
}

#[no_mangle]
pub extern "C" fn rs_op_colon(oap: *mut oparg_T) {
    unsafe { op_colon_c(oap) }
}

#[no_mangle]
pub extern "C" fn rs_op_function(oap: *mut oparg_T) {
    unsafe { op_function_c(oap) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        let res = unsafe { rs_op_change(std::ptr::null_mut()) };
        assert_eq!(res, 0);
    }
}
