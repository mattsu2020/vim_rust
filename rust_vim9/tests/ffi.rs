use std::ffi::CString;

use rust_vim9::vim9_eval_bool;

#[test]
fn ffi_eval_bool() {
    let expr = CString::new("1 < 2").unwrap();
    let res = vim9_eval_bool(expr.as_ptr());
    assert!(res);
}
