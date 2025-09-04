#![allow(clippy::missing_safety_doc)]
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};

#[repr(C)]
pub struct VimDict {
    entries: HashMap<String, *mut c_void>,
}

#[no_mangle]
pub extern "C" fn rust_dict_new() -> *mut VimDict {
    Box::into_raw(Box::new(VimDict {
        entries: HashMap::new(),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn rust_dict_free(d: *mut VimDict) {
    if !d.is_null() {
        drop(Box::from_raw(d));
    }
}

unsafe fn to_key(key: *const c_char) -> Option<String> {
    if key.is_null() {
        None
    } else {
        CStr::from_ptr(key).to_str().ok().map(|s| s.to_owned())
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_dict_set(
    d: *mut VimDict,
    key: *const c_char,
    value: *mut c_void,
) -> c_int {
    let dict = match d.as_mut() {
        Some(d) => d,
        None => return 0,
    };
    let key = match to_key(key) {
        Some(k) => k,
        None => return 0,
    };
    dict.entries.insert(key, value);
    1
}

#[no_mangle]
pub unsafe extern "C" fn rust_dict_get(d: *mut VimDict, key: *const c_char) -> *mut c_void {
    let dict = match d.as_ref() {
        Some(d) => d,
        None => return std::ptr::null_mut(),
    };
    let key = match to_key(key) {
        Some(k) => k,
        None => return std::ptr::null_mut(),
    };
    dict.entries
        .get(&key)
        .copied()
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rust_dict_remove(d: *mut VimDict, key: *const c_char) -> c_int {
    let dict = match d.as_mut() {
        Some(d) => d,
        None => return 0,
    };
    let key = match to_key(key) {
        Some(k) => k,
        None => return 0,
    };
    dict.entries.remove(&key).is_some() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn rust_dict_len(d: *const VimDict) -> usize {
    d.as_ref().map(|d| d.entries.len()).unwrap_or(0)
}
