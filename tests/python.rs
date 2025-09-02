use rust_if_python::{vim_python_init, vim_python_exec};
use std::ffi::CString;

#[test]
fn run_python_code() {
    assert_eq!(vim_python_init(), 1);
    let code = CString::new("x = 40 + 2").unwrap();
    assert_eq!(vim_python_exec(code.as_ptr()), 1);
}
