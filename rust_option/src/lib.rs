use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

static OPTIONS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn options() -> &'static Mutex<HashMap<String, String>> {
    OPTIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[no_mangle]
pub extern "C" fn rs_options_init() {
    let opts = options();
    let mut opts = opts.lock().unwrap();
    if opts.is_empty() {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/sh"));
        opts.insert("shell".into(), shell);
        opts.insert("compatible".into(), "true".into());
    }
}

#[no_mangle]
pub extern "C" fn rs_set_option(name: *const c_char, value: *const c_char) -> bool {
    if name.is_null() || value.is_null() {
        return false;
    }
    let name = unsafe { CStr::from_ptr(name) };
    let value = unsafe { CStr::from_ptr(value) };
    let name = match name.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return false,
    };
    let value = match value.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return false,
    };
    options().lock().unwrap().insert(name, value);
    true
}

#[no_mangle]
pub extern "C" fn rs_get_option(name: *const c_char) -> *mut c_char {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    let name = unsafe { CStr::from_ptr(name) };
    let key = match name.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return std::ptr::null_mut(),
    };
    if let Some(val) = options().lock().unwrap().get(&key) {
        if let Ok(cs) = CString::new(val.as_str()) {
            return cs.into_raw();
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn rs_free_cstring(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options() {
        rs_options_init();
        let name = CString::new("shell").unwrap();
        let val_ptr = rs_get_option(name.as_ptr());
        assert!(!val_ptr.is_null());
        unsafe { drop(CString::from_raw(val_ptr)); }
    }

    #[test]
    fn set_and_get() {
        rs_options_init();
        let name = CString::new("testopt").unwrap();
        let value = CString::new("123").unwrap();
        assert!(rs_set_option(name.as_ptr(), value.as_ptr()));
        let res_ptr = rs_get_option(name.as_ptr());
        assert!(!res_ptr.is_null());
        let res = unsafe { CString::from_raw(res_ptr) };
        assert_eq!(res.to_str().unwrap(), "123");
    }
}

