use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Default)]
struct TextProp {
    id: i32,
    name: String,
}

#[derive(Default)]
struct PropStore {
    props: HashMap<i32, TextProp>,
}

static STORE: OnceLock<Mutex<PropStore>> = OnceLock::new();

fn store() -> &'static Mutex<PropStore> {
    STORE.get_or_init(|| Mutex::new(PropStore::default()))
}

#[no_mangle]
pub extern "C" fn rs_add_text_prop(id: c_int, name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(name) };
    let Ok(name) = cstr.to_str() else { return false }; 
    let mut st = store().lock().unwrap();
    st.props.insert(id, TextProp { id, name: name.to_string() });
    true
}

#[no_mangle]
pub extern "C" fn rs_get_text_prop_name(id: c_int) -> *mut c_char {
    let st = store().lock().unwrap();
    if let Some(tp) = st.props.get(&id) {
        if let Ok(cs) = CString::new(tp.name.as_str()) {
            return cs.into_raw();
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn rs_format_text(
    input: *const c_char,
    uppercase: bool,
    trim: bool,
) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    let cstr = unsafe { CStr::from_ptr(input) };
    let Ok(mut s) = cstr.to_str().map(|s| s.to_string()) else {
        return std::ptr::null_mut();
    };
    if trim {
        s = s.trim().to_string();
    }
    if uppercase {
        s = s.to_uppercase();
    }
    CString::new(s).ok().map_or(std::ptr::null_mut(), |cs| cs.into_raw())
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
    use std::ffi::CString;

    #[test]
    fn add_and_get_prop() {
        let name = CString::new("comment").unwrap();
        assert!(rs_add_text_prop(1, name.as_ptr()));
        let res_ptr = rs_get_text_prop_name(1);
        assert!(!res_ptr.is_null());
        let res = unsafe { CString::from_raw(res_ptr) };
        assert_eq!(res.to_str().unwrap(), "comment");
    }

    #[test]
    fn format_text() {
        let input = CString::new("  hello world  ").unwrap();
        let out_ptr = rs_format_text(input.as_ptr(), true, true);
        assert!(!out_ptr.is_null());
        let out = unsafe { CString::from_raw(out_ptr) };
        assert_eq!(out.to_str().unwrap(), "HELLO WORLD");
    }
}

