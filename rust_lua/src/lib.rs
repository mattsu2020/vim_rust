use mlua::Lua;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

fn run_code(code: *const c_char) -> Result<(), ()> {
    if code.is_null() {
        return Err(());
    }
    let source = unsafe { CStr::from_ptr(code) }.to_str().map_err(|_| ())?;
    let lua = Lua::new();
    lua.load(source).exec().map_err(|_| ())
}

/// Execute Lua code in an embedded interpreter.
#[no_mangle]
pub extern "C" fn vim_lua_exec(code: *const c_char) -> c_int {
    run_code(code).map_or(0, |_| 1)
}

/// Alias exposed for compatibility with existing Vim commands.
#[no_mangle]
pub extern "C" fn lua_execute(code: *const c_char) -> c_int {
    vim_lua_exec(code)
}
