use std::ffi::CString;

use rust_vim9::{vim9_eval_bool, vim9_declare_error_rs};

#[test]
fn ffi_eval_bool() {
    let expr = CString::new("1 < 2").unwrap();
    let res = vim9_eval_bool(expr.as_ptr());
    assert!(res);
}

#[test]
fn ffi_declare_error() {
    let name = CString::new("g:var").unwrap();
    vim9_declare_error_rs(name.as_ptr());
}
