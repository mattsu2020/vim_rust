use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct lua_State {
    _private: [u8; 0],
}

#[link(name = "lua5.4")]
extern "C" {
    fn luaL_newstate() -> *mut lua_State;
    fn luaL_openlibs(L: *mut lua_State);
    fn luaL_loadstring(L: *mut lua_State, s: *const c_char) -> c_int;
    fn lua_pcallk(
        L: *mut lua_State,
        nargs: c_int,
        nresults: c_int,
        errfunc: c_int,
        ctx: isize,
        k: Option<extern "C" fn(*mut lua_State)>,
    ) -> c_int;
    fn lua_close(L: *mut lua_State);
}

#[no_mangle]
pub extern "C" fn vim_lua_exec(code: *const c_char) -> c_int {
    if code.is_null() {
        return 0;
    }
    unsafe {
        let lua_state = luaL_newstate();
        if lua_state.is_null() {
            return 0;
        }
        luaL_openlibs(lua_state);
        let load_status = luaL_loadstring(lua_state, code);
        let result = if load_status == 0 {
            lua_pcallk(lua_state, 0, 0, 0, 0, None)
        } else {
            load_status
        };
        lua_close(lua_state);
        if result == 0 { 1 } else { 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn run_simple_lua() {
        let code = CString::new("x = 2 + 2").unwrap();
        assert_eq!(vim_lua_exec(code.as_ptr()), 1);
    }
}
