use libc::{c_char, c_int};
use std::collections::HashMap;
use std::ffi::CStr;

/// Simple representation of Vim's dict_T using a HashMap of strings to numbers.
#[repr(C)]
pub struct VimDict {
    map: HashMap<String, i64>,
}

impl VimDict {
    fn new() -> Self {
        Self { map: HashMap::new() }
    }
}

/// Allocate a new empty dictionary.
#[no_mangle]
pub extern "C" fn dict_alloc() -> *mut VimDict {
    Box::into_raw(Box::new(VimDict::new()))
}

/// Free a dictionary and all its contents.
#[no_mangle]
pub extern "C" fn dict_free(dict: *mut VimDict) {
    if !dict.is_null() {
        unsafe { drop(Box::from_raw(dict)); }
    }
}

/// Add or replace a number value under `key`.
#[no_mangle]
pub extern "C" fn dict_add_number(dict: *mut VimDict, key: *const c_char, val: i64) -> c_int {
    if dict.is_null() || key.is_null() {
        return 0;
    }
    let d = unsafe { &mut *dict };
    let k = unsafe { CStr::from_ptr(key) }.to_string_lossy().into_owned();
    d.map.insert(k, val as i64);
    1
}

/// Look up a number value by `key`. Returns 1 and stores the value in `out` if
/// found, otherwise returns 0.
#[no_mangle]
pub extern "C" fn dict_lookup_number(dict: *const VimDict, key: *const c_char, out: *mut i64) -> c_int {
    if dict.is_null() || key.is_null() || out.is_null() {
        return 0;
    }
    let d = unsafe { &*dict };
    let k = unsafe { CStr::from_ptr(key) }.to_string_lossy().into_owned();
    if let Some(v) = d.map.get(&k) {
        unsafe { *out = *v as i64; }
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn basic_add_and_lookup() {
        let d = dict_alloc();
        let key = CString::new("answer").unwrap();
        assert_eq!(dict_add_number(d, key.as_ptr(), 42), 1);
        let mut out: i64 = 0;
        assert_eq!(dict_lookup_number(d, key.as_ptr(), &mut out as *mut i64), 1);
        assert_eq!(out, 42);
        dict_free(d);
    }

    #[test]
    fn lookup_missing_key() {
        let d = dict_alloc();
        let key = CString::new("foo").unwrap();
        let mut out: i64 = -1;
        assert_eq!(dict_lookup_number(d, key.as_ptr(), &mut out as *mut i64), 0);
        assert_eq!(out, -1);
        dict_free(d);
    }
}
