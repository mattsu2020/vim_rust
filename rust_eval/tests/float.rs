use std::ffi::CString;

use rust_eval::{eval_expr_rs, eval_to_bool_rs, typval_T, Vartype, ValUnion};

#[test]
fn float_arithmetic() {
    let expr = CString::new("1.5 + 2.5").unwrap();
    let mut out = typval_T { v_type: Vartype::VAR_UNKNOWN, v_lock: 0, vval: ValUnion { v_float: 0.0 } };
    assert!(eval_expr_rs(expr.as_ptr(), &mut out));
    unsafe {
        assert!(matches!(out.v_type, Vartype::VAR_FLOAT));
        assert!((out.vval.v_float - 4.0).abs() < 1e-6);
    }
}

#[test]
fn float_bool() {
    let expr = CString::new("0.0").unwrap();
    let mut err = false;
    let res = eval_to_bool_rs(expr.as_ptr(), &mut err);
    assert!(!res);
    assert!(!err);
}
