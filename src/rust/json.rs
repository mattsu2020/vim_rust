use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Encode a UTF-8 string to a JSON string using serde_json.
#[no_mangle]
pub extern "C" fn json_encode_rs(input: *const c_char) -> *mut c_char {
    unsafe {
        if input.is_null() {
            return CString::new("\"\"").unwrap().into_raw();
        }
        match CStr::from_ptr(input).to_str() {
            Ok(s) => match serde_json::to_string(s) {
                Ok(json) => CString::new(json).unwrap().into_raw(),
                Err(_) => CString::new("\"\"").unwrap().into_raw(),
            },
            Err(_) => CString::new("\"\"").unwrap().into_raw(),
        }
    }
}

// Decode a JSON string back to a plain UTF-8 string.  Returns an empty
// string on error.
#[no_mangle]
pub extern "C" fn json_decode_rs(input: *const c_char) -> *mut c_char {
    unsafe {
        if input.is_null() {
            return CString::new("" ).unwrap().into_raw();
        }
        match CStr::from_ptr(input).to_str() {
            Ok(s) => match serde_json::from_str::<String>(s) {
                Ok(val) => CString::new(val).unwrap().into_raw(),
                Err(_) => CString::new("").unwrap().into_raw(),
            },
            Err(_) => CString::new("").unwrap().into_raw(),
        }
    }
}
