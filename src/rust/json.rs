use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

#[repr(C)]
pub struct JsonResult {
    pub result: *mut c_char,
    pub error: *mut c_char,
}

// Encode a UTF-8 string to a JSON string using serde_json.
#[no_mangle]
pub extern "C" fn json_encode_rs(input: *const c_char) -> JsonResult {
    unsafe {
        if input.is_null() {
            return JsonResult {
                result: ptr::null_mut(),
                error: CString::new("null pointer").unwrap().into_raw(),
            };
        }
        match CStr::from_ptr(input).to_str() {
            Ok(s) => match serde_json::to_string(s) {
                Ok(json) => JsonResult {
                    result: CString::new(json).unwrap().into_raw(),
                    error: ptr::null_mut(),
                },
                Err(e) => JsonResult {
                    result: ptr::null_mut(),
                    error: CString::new(e.to_string()).unwrap().into_raw(),
                },
            },
            Err(_) => JsonResult {
                result: ptr::null_mut(),
                error: CString::new("invalid utf-8").unwrap().into_raw(),
            },
        }
    }
}

// Decode a JSON string back to a plain UTF-8 string.
#[no_mangle]
pub extern "C" fn json_decode_rs(input: *const c_char) -> JsonResult {
    unsafe {
        if input.is_null() {
            return JsonResult {
                result: ptr::null_mut(),
                error: CString::new("null pointer").unwrap().into_raw(),
            };
        }
        match CStr::from_ptr(input).to_str() {
            Ok(s) => match serde_json::from_str::<String>(s) {
                Ok(val) => JsonResult {
                    result: CString::new(val).unwrap().into_raw(),
                    error: ptr::null_mut(),
                },
                Err(e) => JsonResult {
                    result: ptr::null_mut(),
                    error: CString::new(e.to_string()).unwrap().into_raw(),
                },
            },
            Err(_) => JsonResult {
                result: ptr::null_mut(),
                error: CString::new("invalid utf-8").unwrap().into_raw(),
            },
        }
    }
}

// Check whether the JSON input is complete.
// Returns 0 for OK, 1 for FAIL, 2 for MAYBE (incomplete).
#[no_mangle]
pub extern "C" fn json_find_end_rs(input: *const c_char) -> i32 {
    unsafe {
        if input.is_null() {
            return 2; // MAYBE
        }
        match CStr::from_ptr(input).to_str() {
            Ok(s) => match serde_json::from_str::<serde_json::Value>(s) {
                Ok(_) => 0, // OK
                Err(e) => {
                    if e.is_eof() {
                        2 // MAYBE
                    } else {
                        1 // FAIL
                    }
                }
            },
            Err(_) => 1, // FAIL
        }
    }
}
