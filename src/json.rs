use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use serde_json::error::Category;
use serde_json::{self, Value};

// Option flags copied from Vim's C code.  Only JSON_NL has an
// observable effect in this simplified implementation.
const JSON_JS: c_int = 1;
const JSON_NO_NONE: c_int = 2;
const JSON_NL: c_int = 4;

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
    unsafe {
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
                match CString::new(json) {
                    Ok(cjson) => cjson.into_raw(),
                    Err(_) => CString::new("\"\"").unwrap().into_raw(),
                }
            }
            Err(_) => CString::new("\"\"").unwrap().into_raw(),
        }
    }
}

// Decode a JSON value back to a string.  When the decoded value is a JSON
// string the returned text is the plain string, otherwise the JSON
// representation of the value is returned.  On error an empty string is
// returned and "error" indicates whether the input was incomplete (1) or
// invalid (2).  "used" is set to the number of bytes consumed when the
// decoding succeeds.
#[no_mangle]
pub extern "C" fn json_decode_rs(input: *const c_char, _options: c_int) -> JsonDecodeResult {
    unsafe {
        if input.is_null() {
            return JsonDecodeResult {
                ptr: CString::new("").unwrap().into_raw(),
                used: 0,
                error: 2,
            };
        }

        let s = match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => {
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
                let used = de.byte_offset();
                let out = match v {
                    Value::String(st) => st,
                    other => other.to_string(),
                };
                match CString::new(out) {
                    Ok(c_out) => JsonDecodeResult {
                        ptr: c_out.into_raw(),
                        used,
                        error: 0,
                    },
                    Err(_) => JsonDecodeResult {
                        ptr: CString::new("").unwrap().into_raw(),
                        used,
                        error: 2,
                    },
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
}

// Find the end of the first JSON value in "input".  "status" is one of the
// Vim constants OK (1), FAIL (0) or MAYBE (2).
#[no_mangle]
pub extern "C" fn json_find_end_rs(input: *const c_char, _options: c_int) -> JsonFindEndResult {
    unsafe {
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
                let used = de.byte_offset();
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;

    #[test]
    fn encode_handles_inner_null() {
        let bytes = [b'a', 0, b'b', 0];
        let ptr = bytes.as_ptr() as *const c_char;
        let out_ptr = json_encode_rs(ptr, 0);
        unsafe {
            assert_eq!(CStr::from_ptr(out_ptr).to_str().unwrap(), "\"a\"");
            let _ = CString::from_raw(out_ptr);
        }
    }

    #[test]
    fn decode_handles_null_in_output() {
        let input = CString::new("\"a\\u0000b\"").unwrap();
        let res = json_decode_rs(input.as_ptr(), 0);
        unsafe {
            assert_eq!(res.error, 2);
            assert_eq!(CStr::from_ptr(res.ptr).to_bytes(), b"");
            let _ = CString::from_raw(res.ptr);
        }
    }

    #[test]
    fn decode_input_with_null_byte() {
        let bytes = [b'"', b'a', 0, b'b', b'"', 0];
        let res = json_decode_rs(bytes.as_ptr() as *const c_char, 0);
        unsafe {
            assert_eq!(res.error, 1);
            assert_eq!(CStr::from_ptr(res.ptr).to_bytes(), b"");
            let _ = CString::from_raw(res.ptr);
        }
    }
}
