#![allow(clippy::missing_safety_doc)]
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};

/// Internal representation of the hash table. Keys are UTF-8 strings and
/// values are opaque pointers managed by the C side.
type Table = HashMap<String, *mut c_void>;

#[no_mangle]
pub extern "C" fn rust_hashtab_new() -> *mut c_void {
    Box::into_raw(Box::new(Table::new())) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn rust_hashtab_free(tab: *mut c_void) {
    if tab.is_null() {
        return;
    }
    drop(Box::from_raw(tab as *mut Table));
}

unsafe fn to_key(key: *const c_char) -> Option<String> {
    if key.is_null() {
        None
    } else {
        CStr::from_ptr(key).to_str().ok().map(|s| s.to_owned())
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_hashtab_set(
    tab: *mut c_void,
    key: *const c_char,
    value: *mut c_void,
) -> c_int {
    if tab.is_null() {
        return 0;
    }
    let key = match to_key(key) {
        Some(k) => k,
        None => return 0,
    };
    let table = &mut *(tab as *mut Table);
    table.insert(key, value);
    1
}

#[no_mangle]
pub unsafe extern "C" fn rust_hashtab_get(tab: *mut c_void, key: *const c_char) -> *mut c_void {
    if tab.is_null() {
        return std::ptr::null_mut();
    }
    let key = match to_key(key) {
        Some(k) => k,
        None => return std::ptr::null_mut(),
    };
    let table = &*(tab as *mut Table);
    table.get(&key).copied().unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rust_hashtab_remove(tab: *mut c_void, key: *const c_char) -> c_int {
    if tab.is_null() {
        return 0;
    }
    let key = match to_key(key) {
        Some(k) => k,
        None => return 0,
    };
    let table = &mut *(tab as *mut Table);
    table.remove(&key).is_some() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn rust_hashtab_len(tab: *mut c_void) -> usize {
    if tab.is_null() {
        return 0;
    }
    let table = &*(tab as *mut Table);
    table.len()
}
