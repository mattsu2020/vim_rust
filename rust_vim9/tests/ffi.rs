use std::ffi::CString;

use rust_vim9::{
    vim9_eval_bool, vim9_eval_int, vim9_exec_rs, vim9_declare_error_rs,
};
use rust_eval::{typval_T, ValUnion, Vartype};

#[test]
fn ffi_eval_bool() {
    let expr = CString::new("1 < 2").unwrap();
    let res = vim9_eval_bool(expr.as_ptr());
    assert!(res);
}

#[test]
fn ffi_eval_int() {
    let expr = CString::new("1 + 2 + 3").unwrap();
    let res = vim9_eval_int(expr.as_ptr());
    assert_eq!(res, 6);
}

#[test]
fn ffi_exec_expression() {
    let expr = CString::new("echo 4 + 5").unwrap();
    let mut out = typval_T {
        v_type: Vartype::VAR_UNKNOWN,
        v_lock: 0,
        vval: ValUnion { v_number: 0 },
    };
    let ok = vim9_exec_rs(expr.as_ptr(), &mut out as *mut typval_T);
    assert!(ok);
    unsafe { assert_eq!(out.vval.v_number, 9); }
}

#[test]
fn ffi_declare_error() {
    let name = CString::new("g:var").unwrap();
    vim9_declare_error_rs(name.as_ptr());
}
