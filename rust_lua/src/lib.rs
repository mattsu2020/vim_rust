use mlua::Lua;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// Execute Lua code in an embedded interpreter.
#[no_mangle]
pub extern "C" fn vim_lua_exec(code: *const c_char) -> c_int {
    if code.is_null() {
        return 0;
    }
    let c_str = unsafe { CStr::from_ptr(code) };
    let source = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let lua = Lua::new();
    match lua.load(source).exec() {
        Ok(_) => 1,
        Err(_) => 0,
    }
}
