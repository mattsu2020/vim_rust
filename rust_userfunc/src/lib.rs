use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::sync::{Mutex, OnceLock};

use hashbrown::HashMap;

#[repr(C)]
pub struct rust_funccall_S {
    pub previous: *mut rust_funccall_S,
    pub depth: c_int,
}

#[repr(C)]
struct FuncState {
    table: HashMap<String, *mut c_void>,
    current: *mut rust_funccall_S,
}

unsafe impl Send for FuncState {}

static FUNC_STATE: OnceLock<Mutex<FuncState>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn rust_func_init() {
    FUNC_STATE.get_or_init(|| {
        Mutex::new(FuncState {
            table: HashMap::new(),
            current: std::ptr::null_mut(),
        })
    });
}

#[no_mangle]
pub extern "C" fn rust_func_deinit() {
    if let Some(state) = FUNC_STATE.get() {
        if let Ok(mut guard) = state.lock() {
            guard.table.clear();
            guard.current = std::ptr::null_mut();
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

fn with_state<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut FuncState) -> R,
{
    let state = FUNC_STATE.get()?;
    if let Ok(mut guard) = state.lock() {
        Some(f(&mut *guard))
    } else {
        None
    }
}

#[no_mangle]
pub extern "C" fn rust_func_hashtab_set(name: *const c_char, func: *mut c_void) -> c_int {
    if name.is_null() {
        return 0;
    }
    with_state(|st| {
        let key = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        st.table.insert(key, func);
        1
    })
    .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn rust_func_hashtab_get(name: *const c_char) -> *mut c_void {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    with_state(|st| {
        let key = unsafe { CStr::from_ptr(name) }.to_string_lossy();
        st.table
            .get(key.as_ref())
            .copied()
            .unwrap_or(std::ptr::null_mut())
    })
    .unwrap_or(std::ptr::null_mut())
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
