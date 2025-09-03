use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

static GROUPS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Define a highlight group identified by name with a simple attribute string.
#[no_mangle]
pub extern "C" fn rs_highlight_define(name: *const c_char, attrs: *const c_char) {
    let Ok(name) = unsafe { CStr::from_ptr(name) }.to_str().map(|s| s.to_owned()) else {
        return;
    };
    let Ok(attrs) = unsafe { CStr::from_ptr(attrs) }.to_str().map(|s| s.to_owned()) else {
        return;
    };
    GROUPS.lock().unwrap().insert(name, attrs);
}

/// Query attributes for a highlight group.
/// Returns a newly allocated C string which must be freed by the caller using
/// `libc::free`.
#[no_mangle]
pub extern "C" fn rs_highlight_get(name: *const c_char) -> *mut c_char {
    let Ok(name) = unsafe { CStr::from_ptr(name) }.to_str().map(|s| s.to_owned()) else {
        return std::ptr::null_mut();
    };
    let Some(attrs) = GROUPS.lock().unwrap().get(&name).cloned() else {
        return std::ptr::null_mut();
    };
    let cstring = match CString::new(attrs) {
        Ok(c) => c,
        Err(_) => return std::ptr::null_mut(),
    };
    cstring.into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn define_and_get() {
        let name = CString::new("MyGroup").unwrap();
        let attrs = CString::new("fg=red,bg=blue").unwrap();
        rs_highlight_define(name.as_ptr(), attrs.as_ptr());
        let res_ptr = rs_highlight_get(name.as_ptr());
        let res = unsafe { CString::from_raw(res_ptr) };
        assert_eq!(res.to_str().unwrap(), "fg=red,bg=blue");
    }
}
