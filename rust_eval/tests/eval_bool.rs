use std::ffi::CString;

use rust_eval::eval_to_bool_rs;

#[test]
fn eval_bool_true() {
    let expr = CString::new("1 + 1").unwrap();
    let mut err = false;
    let res = eval_to_bool_rs(expr.as_ptr(), &mut err);
    assert!(res);
    assert!(!err);
}

#[test]
fn eval_bool_error() {
    let expr = CString::new("1 + ").unwrap();
    let mut err = false;
    let res = eval_to_bool_rs(expr.as_ptr(), &mut err);
    assert!(!res);
    assert!(err);
}
