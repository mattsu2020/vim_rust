use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use serde::Deserialize;
use serde_json::error::Category;
use serde_json::{self, Value};

// Option flags copied from Vim's C code.  Only JSON_NL has an
// observable effect in this simplified implementation.
const JSON_JS: c_int = 1;
const JSON_NO_NONE: c_int = 2;
const JSON_NL: c_int = 4;

fn cstr_to_str<'a>(input: *const c_char) -> Option<&'a str> {
    if input.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(input).to_str().ok() }
}

/// Result for decoding a JSON string.
#[repr(C)]
pub struct JsonDecodeResult {
    pub ptr: *mut c_char,
    pub used: usize,
    pub error: c_int,
}

/// Result for finding the end of a JSON message.
#[repr(C)]
pub struct JsonFindEndResult {
    pub used: usize,
    pub status: c_int,
}

// Encode a UTF-8 string to a JSON string using serde_json.  The only option
// currently honored is JSON_NL, which appends a trailing newline to the
// encoded result when set.
#[no_mangle]
pub extern "C" fn json_encode_rs(input: *const c_char, options: c_int) -> *mut c_char {
    let s = match cstr_to_str(input) {
        Some(s) => s,
        None => return CString::new("\"\"").unwrap().into_raw(),
    };

    let mut json = match serde_json::to_string(s) {
        Ok(json) => json,
        Err(_) => "\"\"".to_string(),
    };
    if (options & JSON_NL) != 0 {
        json.push('\n');
    }
    CString::new(json).unwrap().into_raw()
}

// Decode a JSON value back to a string.  When the decoded value is a JSON
// string the returned text is the plain string, otherwise the JSON
// representation of the value is returned.  On error an empty string is
// returned and "error" indicates whether the input was incomplete (1) or
// invalid (2).  "used" is set to the number of bytes consumed when the
// decoding succeeds.
#[no_mangle]
pub extern "C" fn json_decode_rs(input: *const c_char, _options: c_int) -> JsonDecodeResult {
    let s = match cstr_to_str(input) {
        Some(s) => s,
        None => {
            return JsonDecodeResult {
                ptr: CString::new("").unwrap().into_raw(),
                used: 0,
                error: 2,
            };
        }
    };

    let mut de = serde_json::Deserializer::from_str(s);
    match Value::deserialize(&mut de) {
        Ok(v) => {
            let used = s.len();
            let out = match v {
                Value::String(st) => st,
                other => other.to_string(),
            };
            JsonDecodeResult {
                ptr: CString::new(out).unwrap().into_raw(),
                used,
                error: 0,
            }
        }
        Err(e) => {
            let err = match e.classify() {
                Category::Eof => 1,
                _ => 2,
            };
            JsonDecodeResult {
                ptr: CString::new("").unwrap().into_raw(),
                used: 0,
                error: err,
            }
        }
    }
}

// Find the end of the first JSON value in "input".  "status" is one of the
// Vim constants OK (1), FAIL (0) or MAYBE (2).
#[no_mangle]
pub extern "C" fn json_find_end_rs(input: *const c_char, _options: c_int) -> JsonFindEndResult {
    let s = match cstr_to_str(input) {
        Some(s) => s,
        None => return JsonFindEndResult { used: 0, status: 0 },
    };

    let mut de = serde_json::Deserializer::from_str(s);
    match Value::deserialize(&mut de) {
        Ok(_) => {
            let used = s.len();
            JsonFindEndResult { used, status: 1 }
        }
        Err(e) => {
            let status = match e.classify() {
                Category::Eof => 2,
                _ => 0,
            };
            JsonFindEndResult { used: 0, status }
        }
    }
}
