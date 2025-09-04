use std::ffi::CString;

use rust_eval::{eval_script_rs, eval_variable_rs, typval_T, Vartype, ValUnion};

#[test]
fn script_assignment_and_eval() {
    let script = CString::new("let x = 1\nx + 2").unwrap();
    let mut out = typval_T { v_type: Vartype::VAR_UNKNOWN, v_lock: 0, vval: ValUnion { v_number: 0 } };
    assert!(eval_script_rs(script.as_ptr(), &mut out));
    unsafe { assert_eq!(out.vval.v_number, 3); }
    let name = CString::new("x").unwrap();
    let mut var = typval_T { v_type: Vartype::VAR_UNKNOWN, v_lock: 0, vval: ValUnion { v_number: 0 } };
    assert!(eval_variable_rs(name.as_ptr(), &mut var));
    unsafe { assert_eq!(var.vval.v_number, 1); }
}

#[test]
fn script_float() {
    let script = CString::new("let f = 1.5\nf * 2").unwrap();
    let mut out = typval_T { v_type: Vartype::VAR_UNKNOWN, v_lock: 0, vval: ValUnion { v_float: 0.0 } };
    assert!(eval_script_rs(script.as_ptr(), &mut out));
    unsafe {
        assert!(matches!(out.v_type, Vartype::VAR_FLOAT));
        assert!((out.vval.v_float - 3.0).abs() < 1e-6);
    }
}
