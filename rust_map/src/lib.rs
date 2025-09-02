use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

static MAPS: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[no_mangle]
pub extern "C" fn rs_map_add(lhs: *const c_char, rhs: *const c_char) {
    let lhs = unsafe { CStr::from_ptr(lhs) }.to_string_lossy().into_owned();
    let rhs = unsafe { CStr::from_ptr(rhs) }.to_string_lossy().into_owned();
    MAPS.lock().unwrap().insert(lhs, rhs);
}

#[no_mangle]
pub extern "C" fn rs_map_lookup(lhs: *const c_char) -> *const c_char {
    let lhs = unsafe { CStr::from_ptr(lhs) }.to_str().unwrap_or("");
    if let Some(rhs) = MAPS.lock().unwrap().get(lhs) {
        let cstr = CString::new(rhs.clone()).unwrap();
        cstr.into_raw()
    } else {
        std::ptr::null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_lookup() {
        rs_map_add(b"jj\0".as_ptr() as *const c_char, b"<Esc>\0".as_ptr() as *const c_char);
        let ptr = rs_map_lookup(b"jj\0".as_ptr() as *const c_char);
        let cstr = unsafe { CStr::from_ptr(ptr) };
        assert_eq!(cstr.to_str().unwrap(), "<Esc>");
    }
}
