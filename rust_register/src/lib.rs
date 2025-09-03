use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::{Mutex, OnceLock};

static REGISTERS: OnceLock<Mutex<HashMap<char, String>>> = OnceLock::new();

fn registers() -> &'static Mutex<HashMap<char, String>> {
    REGISTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn set_register(reg: char, val: &str) {
    if reg == '*' || reg == '+' {
        let _ = rust_clipboard::set_string(val);
    } else {
        registers().lock().unwrap().insert(reg, val.to_string());
    }
}

pub fn get_register(reg: char) -> Option<String> {
    if reg == '*' || reg == '+' {
        rust_clipboard::get_string()
    } else {
        registers().lock().unwrap().get(&reg).cloned()
    }
}

#[no_mangle]
pub extern "C" fn rs_register_set(reg: c_char, value: *const c_char) -> c_int {
    if value.is_null() {
        return -1;
    }
    let reg = reg as u8 as char;
    let cstr = unsafe { CStr::from_ptr(value) };
    let Ok(s) = cstr.to_str() else { return -1 };
    set_register(reg, s);
    0
}

#[no_mangle]
pub extern "C" fn rs_register_get(reg: c_char) -> *mut c_char {
    let reg = reg as u8 as char;
    if let Some(s) = get_register(reg) {
        CString::new(s).unwrap().into_raw()
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn rs_register_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn set_and_get_register() {
        let reg = 'a';
        let value = "hello";
        let cval = CString::new(value).unwrap();
        assert_eq!(rs_register_set(reg as c_char, cval.as_ptr()), 0);
        let ptr = rs_register_get(reg as c_char);
        assert!(!ptr.is_null());
        let s = unsafe { CString::from_raw(ptr) };
        assert_eq!(s.to_str().unwrap(), value);
    }

    #[test]
    fn clipboard_register() {
        let reg = '*';
        let cval = CString::new("clip").unwrap();
        assert_eq!(rs_register_set(reg as c_char, cval.as_ptr()), 0);
        let ptr = rs_register_get(reg as c_char);
        assert!(!ptr.is_null());
        let s = unsafe { CString::from_raw(ptr) };
        assert_eq!(s.to_str().unwrap(), "clip");
    }
}
