use std::ffi::CString;

use rust_eval::{eval_variable_rs, set_variable_rs, typval_T, Vartype, ValUnion};

#[test]
fn variable_roundtrip() {
    let name = CString::new("x").unwrap();
    let val = typval_T { v_type: Vartype::VAR_NUMBER, v_lock: 0, vval: ValUnion { v_number: 7 } };
    assert!(set_variable_rs(name.as_ptr(), &val));
    let mut out = typval_T { v_type: Vartype::VAR_UNKNOWN, v_lock: 0, vval: ValUnion { v_number: 0 } };
    assert!(eval_variable_rs(name.as_ptr(), &mut out));
    unsafe { assert_eq!(out.vval.v_number, 7); }
}
