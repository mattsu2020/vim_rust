#![allow(clippy::missing_safety_doc)]
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use serde::Deserialize;
use serde_json::error::Category;
use serde_json::{self, Value};

const JSON_JS: c_int = 1;
const JSON_NO_NONE: c_int = 2;
const JSON_NL: c_int = 4;

#[repr(C)]
pub struct JsonDecodeResult {
    pub ptr: *mut c_char,
    pub used: usize,
    pub error: c_int,
}

#[repr(C)]
pub struct JsonFindEndResult {
    pub used: usize,
    pub status: c_int,
}

#[no_mangle]
pub unsafe extern "C" fn json_encode_rs(input: *const c_char, options: c_int) -> *mut c_char {
    if input.is_null() {
        return CString::new("\"\"").unwrap().into_raw();
    }
    match CStr::from_ptr(input).to_str() {
        Ok(s) => {
            let mut json = match serde_json::to_string(s) {
                Ok(json) => json,
                Err(_) => "\"\"".to_string(),
            };
            if (options & JSON_NL) != 0 {
                json.push('\n');
            }
            CString::new(json).unwrap().into_raw()
        }
        Err(_) => CString::new("\"\"").unwrap().into_raw(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn json_decode_rs(input: *const c_char, _options: c_int) -> JsonDecodeResult {
    if input.is_null() {
        return JsonDecodeResult { ptr: CString::new("").unwrap().into_raw(), used: 0, error: 2 };
    }
    let s = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => {
            return JsonDecodeResult { ptr: CString::new("").unwrap().into_raw(), used: 0, error: 2 };
        }
    };

    let mut de = serde_json::Deserializer::from_str(s);
    match Value::deserialize(&mut de) {
        Ok(v) => {
            let used = s.len();
            let out = match v { Value::String(st) => st, other => other.to_string() };
            JsonDecodeResult { ptr: CString::new(out).unwrap().into_raw(), used, error: 0 }
        }
        Err(e) => {
            let err = match e.classify() { Category::Eof => 1, _ => 2 };
            JsonDecodeResult { ptr: CString::new("").unwrap().into_raw(), used: 0, error: err }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn json_find_end_rs(input: *const c_char, _options: c_int) -> JsonFindEndResult {
    if input.is_null() {
        return JsonFindEndResult { used: 0, status: 0 };
    }
    let s = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return JsonFindEndResult { used: 0, status: 0 },
    };

    let mut de = serde_json::Deserializer::from_str(s);
    match Value::deserialize(&mut de) {
        Ok(_) => {
                JsonFindEndResult { used: s.len(), status: 1 }
        }
        Err(e) => {
            let status = match e.classify() { Category::Eof => 2, _ => 0 };
            JsonFindEndResult { used: 0, status }
        }
    }
}
