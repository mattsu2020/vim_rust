use rust_if_lua::vim_lua_exec;
use std::ffi::CString;

#[test]
fn run_lua_code() {
    let code = CString::new("x = 40 + 2").unwrap();
    assert_eq!(vim_lua_exec(code.as_ptr()), 1);
}
