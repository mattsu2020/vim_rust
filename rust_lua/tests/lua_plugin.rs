use std::ffi::CString;
use rust_lua::{vim_lua_exec, vim_lua_init, vim_lua_end};

#[test]
fn run_sample_lua_plugin() {
    let code = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/sample_lua_plugin.lua")).unwrap();
    let cstr = CString::new(code).unwrap();
    assert_eq!(vim_lua_init(), 1);
    assert_eq!(vim_lua_exec(cstr.as_ptr()), 1);
    assert_eq!(vim_lua_end(), 1);
}
