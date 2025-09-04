use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

fn registry() -> &'static Mutex<HashMap<String, String>> {
    static REG: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn define_user_command(name: &str, expansion: &str) {
    registry().lock().unwrap().insert(name.to_string(), expansion.to_string());
}

pub fn expand_user_command(name: &str) -> Option<String> {
    registry().lock().unwrap().get(name).cloned()
}

#[no_mangle]
pub extern "C" fn rs_define_user_command(name: *const c_char, expansion: *const c_char) -> bool {
    if name.is_null() || expansion.is_null() {
        return false;
    }
    let name = unsafe { CStr::from_ptr(name) };
    let expansion = unsafe { CStr::from_ptr(expansion) };
    let (Ok(name), Ok(expansion)) = (name.to_str(), expansion.to_str()) else { return false };
    define_user_command(name, expansion);
    true
}

#[no_mangle]
pub extern "C" fn rs_expand_user_command(name: *const c_char) -> *mut c_char {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    let name = unsafe { CStr::from_ptr(name) };
    let Ok(name) = name.to_str() else { return std::ptr::null_mut() };
    match expand_user_command(name) {
        Some(s) => CString::new(s).unwrap().into_raw(),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rs_usercmd_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn define_and_expand() {
        define_user_command("Foo", "cmd1");
        assert_eq!(expand_user_command("Foo").unwrap(), "cmd1");
    }
}

