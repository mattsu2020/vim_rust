use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

/// Call stack entry for a user-defined function.
#[repr(C)]
pub struct Funccall {
    /// Previous entry in the call stack.
    pub previous: *mut Funccall,
    /// Depth of the call.
    pub depth: i32,
}

struct FuncState {
    table: HashMap<String, usize>,
    current: *mut Funccall,
}

unsafe impl Send for FuncState {}
unsafe impl Send for Funccall {}

static FUNC_STATE: OnceLock<Mutex<FuncState>> = OnceLock::new();

/// Initialise the global function table.
pub fn func_init() {
    FUNC_STATE.get_or_init(|| {
        Mutex::new(FuncState {
            table: HashMap::new(),
            current: std::ptr::null_mut(),
        })
    });
}

/// Clear the function table and reset the current call stack.
pub fn func_deinit() {
    if let Some(state) = FUNC_STATE.get() {
        if let Ok(mut guard) = state.lock() {
            guard.table.clear();
            guard.current = std::ptr::null_mut();
        }
    }
}

/// Allocate a new [`Funccall`] entry.
pub fn funccall_new(previous: *mut Funccall, depth: i32) -> *mut Funccall {
    Box::into_raw(Box::new(Funccall { previous, depth }))
}

/// Free a [`Funccall`] entry previously allocated with [`funccall_new`].
pub fn funccall_free(fc: *mut Funccall) {
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
    Some(f(&mut *state.lock().ok()?))
}

/// Store a function pointer under `name`.
/// Returns `true` on success.
pub fn func_hashtab_set(name: &str, func: *mut c_void) -> bool {
    if name.is_empty() {
        return false;
    }
    with_state(|st| {
        st.table.insert(name.to_string(), func as usize);
        true
    })
    .unwrap_or(false)
}

/// Retrieve a previously stored function pointer by name.
pub fn func_hashtab_get(name: &str) -> *mut c_void {
    if name.is_empty() {
        return std::ptr::null_mut();
    }
    let ptr = with_state(|st| st.table.get(name).copied().unwrap_or(0)).unwrap_or(0);
    ptr as *mut c_void
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_get() {
        func_init();
        let fptr = 0xdeadbeef as *mut c_void;
        assert!(func_hashtab_set("foo", fptr));
        assert_eq!(func_hashtab_get("foo"), fptr);
        func_deinit();
    }

    #[test]
    fn funccall_alloc_free() {
        let fc = funccall_new(std::ptr::null_mut(), 1);
        assert!(!fc.is_null());
        funccall_free(fc);
    }
}

