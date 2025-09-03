use std::os::raw::{c_char, c_int, c_void};
use std::sync::Mutex;

use rust_hashtab::{
    rust_hashtab_free, rust_hashtab_get, rust_hashtab_new, rust_hashtab_set,
};

#[repr(C)]
pub struct rust_funccall_S {
    pub previous: *mut rust_funccall_S,
    pub depth: c_int,
}

#[repr(C)]
struct FuncState {
    table: *mut c_void,
    current: *mut rust_funccall_S,
}

static mut FUNC_STATE: *mut Mutex<FuncState> = std::ptr::null_mut();

#[no_mangle]
pub extern "C" fn rust_func_init() {
    let state = Mutex::new(FuncState {
        table: rust_hashtab_new(),
        current: std::ptr::null_mut(),
    });
    unsafe {
        if FUNC_STATE.is_null() {
            FUNC_STATE = Box::into_raw(Box::new(state));
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_func_deinit() {
    unsafe {
        if !FUNC_STATE.is_null() {
            if let Ok(guard) = (*FUNC_STATE).lock() {
                rust_hashtab_free(guard.table);
            }
            drop(Box::from_raw(FUNC_STATE));
            FUNC_STATE = std::ptr::null_mut();
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_funccall_new(
    previous: *mut rust_funccall_S,
    depth: c_int,
) -> *mut rust_funccall_S {
    Box::into_raw(Box::new(rust_funccall_S { previous, depth }))
}

#[no_mangle]
pub extern "C" fn rust_funccall_free(fc: *mut rust_funccall_S) {
    if !fc.is_null() {
        unsafe {
            drop(Box::from_raw(fc));
        }
    }
}

unsafe fn with_state<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut FuncState) -> R,
{
    if FUNC_STATE.is_null() {
        return None;
    }
    let state = &mut *FUNC_STATE;
    if let Ok(mut guard) = state.lock() {
        Some(f(&mut *guard))
    } else {
        None
    }
}

#[no_mangle]
pub extern "C" fn rust_func_hashtab_set(
    name: *const c_char,
    func: *mut c_void,
) -> c_int {
    if name.is_null() {
        return 0;
    }
    unsafe { with_state(|st| rust_hashtab_set(st.table, name, func)) }
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn rust_func_hashtab_get(name: *const c_char) -> *mut c_void {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { with_state(|st| rust_hashtab_get(st.table, name)).unwrap_or(std::ptr::null_mut()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn store_and_get() {
        rust_func_init();
        let name = CString::new("foo").unwrap();
        let fptr = 0xdeadbeef as *mut c_void;
        assert_eq!(rust_func_hashtab_set(name.as_ptr(), fptr), 1);
        assert_eq!(rust_func_hashtab_get(name.as_ptr()), fptr);
        rust_func_deinit();
    }

    #[test]
    fn funccall_alloc_free() {
        let fc = rust_funccall_new(std::ptr::null_mut(), 1);
        assert!(!fc.is_null());
        rust_funccall_free(fc);
    }
}
