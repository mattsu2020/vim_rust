use libc::{c_char, c_int, size_t};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::Mutex;

static MARKERS: Lazy<Mutex<HashMap<String, usize>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[no_mangle]
pub extern "C" fn rust_set_marker(name: *const c_char, pos: size_t) {
    if name.is_null() {
        return;
    }
    let cstr = unsafe { CStr::from_ptr(name) };
    if let Ok(s) = cstr.to_str() {
        MARKERS.lock().unwrap().insert(s.to_string(), pos as usize);
    }
}

#[no_mangle]
pub extern "C" fn rust_get_marker(name: *const c_char, pos_out: *mut size_t) -> c_int {
    if name.is_null() || pos_out.is_null() {
        return 0;
    }
    let cstr = unsafe { CStr::from_ptr(name) };
    if let Ok(s) = cstr.to_str() {
        if let Some(&p) = MARKERS.lock().unwrap().get(s) {
            unsafe { *pos_out = p as size_t; }
            return 1;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn marker_roundtrip() {
        let name = CString::new("A").unwrap();
        rust_set_marker(name.as_ptr(), 42);
        let mut out: size_t = 0;
        assert_eq!(rust_get_marker(name.as_ptr(), &mut out as *mut size_t), 1);
        assert_eq!(out, 42);
    }

    #[test]
    fn missing_marker() {
        let name = CString::new("Z").unwrap();
        let mut out: size_t = 0;
        assert_eq!(rust_get_marker(name.as_ptr(), &mut out as *mut size_t), 0);
    }
}
