use mlua::Lua;
use std::os::raw::{c_char, c_int};

use rust_ffi::{cstr_to_str, result_to_int, FFIError, FFIResult};

fn run_code(code: *const c_char) -> FFIResult<()> {
    let source = cstr_to_str(code)?;
    let lua = Lua::new();
    lua.load(source).exec().map_err(|_| FFIError::Exec)
}

/// Execute Lua code in an embedded interpreter.
#[no_mangle]
pub extern "C" fn vim_lua_exec(code: *const c_char) -> c_int {
    result_to_int(run_code(code))
}

/// Alias exposed for compatibility with existing Vim commands.
#[no_mangle]
pub extern "C" fn lua_execute(code: *const c_char) -> c_int {
    vim_lua_exec(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn exec_and_null_error() {
        let code = CString::new("return 1").unwrap();
        assert_eq!(vim_lua_exec(code.as_ptr()), 1);
        assert_eq!(vim_lua_exec(std::ptr::null()), 0);
    }
}
