use mlua::Lua;
use once_cell::unsync::OnceCell;
use std::cell::RefCell;
use std::os::raw::{c_char, c_int};

use rust_ffi::{cstr_to_str, result_to_int, FFIError, FFIResult};

thread_local! {
    static LUA: RefCell<OnceCell<Lua>> = RefCell::new(OnceCell::new());
}

fn init() -> FFIResult<()> {
    LUA.with(|cell| {
        let mut cell = cell.borrow_mut();
        if cell.get().is_none() {
            cell.set(Lua::new()).map_err(|_| FFIError::Init)?;
        }
        Ok(())
    })
}

fn end() -> FFIResult<()> {
    LUA.with(|cell| {
        cell.borrow_mut().take();
    });
    Ok(())
}

fn run_code(code: *const c_char) -> FFIResult<()> {
    let source = cstr_to_str(code)?;
    LUA.with(|cell| {
        let mut cell = cell.borrow_mut();
        let lua = cell.get_or_init(Lua::new);
        lua.load(source).exec().map_err(|_| FFIError::Exec)
    })
}

/// Execute Lua code in an embedded interpreter.
#[no_mangle]
pub extern "C" fn vim_lua_exec(code: *const c_char) -> c_int {
    result_to_int(run_code(code))
}

/// Initialize the embedded Lua interpreter.
#[no_mangle]
pub extern "C" fn vim_lua_init() -> c_int {
    result_to_int(init())
}

/// Finalize the embedded Lua interpreter.
#[no_mangle]
pub extern "C" fn vim_lua_end() -> c_int {
    result_to_int(end())
}

/// Alias exposed for compatibility with existing Vim commands.
#[no_mangle]
pub extern "C" fn lua_execute(code: *const c_char) -> c_int {
    vim_lua_exec(code)
}

/// Alias to initialize Lua via a traditional name.
#[no_mangle]
pub extern "C" fn lua_init() -> c_int {
    vim_lua_init()
}

/// Alias to finalize Lua via a traditional name.
#[no_mangle]
pub extern "C" fn lua_end() -> c_int {
    vim_lua_end()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn exec_and_null_error() {
        assert_eq!(vim_lua_init(), 1);
        let code = CString::new("return 1").unwrap();
        assert_eq!(vim_lua_exec(code.as_ptr()), 1);
        assert_eq!(vim_lua_exec(std::ptr::null()), 0);
        assert_eq!(vim_lua_end(), 1);
    }
}
