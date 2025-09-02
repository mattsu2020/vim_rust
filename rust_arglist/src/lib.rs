use std::ffi::{CStr};
use std::os::raw::{c_char, c_int};
use std::sync::{Mutex, OnceLock};

#[repr(C)]
pub struct ArgList {
    args: Vec<String>,
}

impl ArgList {
    fn new() -> Self {
        Self { args: Vec::new() }
    }
}

static ARGLIST_LOCKED: OnceLock<Mutex<bool>> = OnceLock::new();

fn lock_state() -> &'static Mutex<bool> {
    ARGLIST_LOCKED.get_or_init(|| Mutex::new(false))
}

#[no_mangle]
pub extern "C" fn rs_arglist_new() -> *mut ArgList {
    Box::into_raw(Box::new(ArgList::new()))
}

#[no_mangle]
pub extern "C" fn rs_arglist_add(al: *mut ArgList, arg: *const c_char) {
    if al.is_null() || arg.is_null() {
        return;
    }
    let c_str = unsafe { CStr::from_ptr(arg) };
    let list = unsafe { &mut *al };
    list.args.push(c_str.to_string_lossy().into_owned());
}

#[no_mangle]
pub extern "C" fn rs_arglist_len(al: *const ArgList) -> usize {
    if al.is_null() {
        return 0;
    }
    let list = unsafe { &*al };
    list.args.len()
}

#[no_mangle]
pub extern "C" fn rs_arglist_free(al: *mut ArgList) {
    if !al.is_null() {
        unsafe { drop(Box::from_raw(al)); }
    }
}

#[no_mangle]
pub extern "C" fn rs_arglist_locked() -> c_int {
    if let Ok(locked) = lock_state().lock() {
        if *locked { 1 } else { 0 }
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn rs_arglist_lock() {
    let m = lock_state();
    let mut guard = m.lock().unwrap();
    *guard = true;
}

#[no_mangle]
pub extern "C" fn rs_arglist_unlock() {
    if let Some(m) = ARGLIST_LOCKED.get() {
        let mut guard = m.lock().unwrap();
        *guard = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn basic_operations() {
        let list = rs_arglist_new();
        let arg = CString::new("foo").unwrap();
        rs_arglist_add(list, arg.as_ptr());
        assert_eq!(rs_arglist_len(list), 1);
        rs_arglist_free(list);
    }

    #[test]
    fn lock_control() {
        rs_arglist_unlock(); // ensure unlocked
        rs_arglist_lock();
        assert_eq!(rs_arglist_locked(), 1);
        rs_arglist_unlock();
        assert_eq!(rs_arglist_locked(), 0);
    }
}
