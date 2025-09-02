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
pub extern "C" fn rust_hashtab_free(tab: *mut c_void) {
    if tab.is_null() {
        return;
    }
    // Reconstruct the Box to drop the table and free its memory.
    unsafe { drop(Box::from_raw(tab as *mut Table)); }
}

unsafe fn to_key(key: *const c_char) -> Option<String> {
    if key.is_null() {
        return None;
    }
    CStr::from_ptr(key).to_str().ok().map(|s| s.to_owned())
}

#[no_mangle]
pub extern "C" fn rust_hashtab_set(
    tab: *mut c_void,
    key: *const c_char,
    value: *mut c_void,
) -> c_int {
    if tab.is_null() {
        return 0;
    }
    let key = match unsafe { to_key(key) } {
        Some(k) => k,
        None => return 0,
    };
    let table = unsafe { &mut *(tab as *mut Table) };
    table.insert(key, value);
    1
}

#[no_mangle]
pub extern "C" fn rust_hashtab_get(
    tab: *mut c_void,
    key: *const c_char,
) -> *mut c_void {
    if tab.is_null() {
        return std::ptr::null_mut();
    }
    let key = match unsafe { to_key(key) } {
        Some(k) => k,
        None => return std::ptr::null_mut(),
    };
    let table = unsafe { &*(tab as *mut Table) };
    table.get(&key).copied().unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn rust_hashtab_remove(
    tab: *mut c_void,
    key: *const c_char,
) -> c_int {
    if tab.is_null() {
        return 0;
    }
    let key = match unsafe { to_key(key) } {
        Some(k) => k,
        None => return 0,
    };
    let table = unsafe { &mut *(tab as *mut Table) };
    table.remove(&key).is_some() as c_int
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn basic_operations() {
        let tab = rust_hashtab_new();
        let key = CString::new("alpha").unwrap();
        let value = 0xdeadbeef as *mut c_void;

        assert_eq!(rust_hashtab_get(tab, key.as_ptr()), std::ptr::null_mut());
        assert_eq!(rust_hashtab_set(tab, key.as_ptr(), value), 1);
        assert_eq!(rust_hashtab_get(tab, key.as_ptr()), value);
        assert_eq!(rust_hashtab_remove(tab, key.as_ptr()), 1);
        assert_eq!(rust_hashtab_get(tab, key.as_ptr()), std::ptr::null_mut());
        rust_hashtab_free(tab);
    }
}
