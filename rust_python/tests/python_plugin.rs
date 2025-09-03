use std::ffi::CString;
use rust_python::{vim_python_init, vim_python_exec, vim_python_end};

#[test]
fn run_sample_python_plugin() {
    let code = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/sample_python_plugin.py")).unwrap();
    let cstr = CString::new(code).unwrap();
    assert_eq!(vim_python_init(), 1);
    assert_eq!(vim_python_exec(cstr.as_ptr()), 1);
    assert_eq!(vim_python_end(), 1);
}
