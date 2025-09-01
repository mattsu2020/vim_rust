use libc::{c_char, c_int, c_longlong};
use serde_json::{self, Value};
use std::ffi::{CStr, CString};

#[repr(C)]
pub union vval_u {
    v_number: c_longlong,
    v_string: *const c_char,
}

#[repr(C)]
pub struct typval_T {
    pub v_type: c_int,
    pub vval: vval_u,
}

const VAR_NUMBER: c_int = 0;
const VAR_STRING: c_int = 1;

#[no_mangle]
pub unsafe extern "C" fn rs_json_encode(tv: *const typval_T, _flags: c_int) -> *mut c_char {
    if tv.is_null() {
        return CString::new("null").unwrap().into_raw();
    }
    let s = match (*tv).v_type {
        VAR_NUMBER => serde_json::to_string(&(*tv).vval.v_number).unwrap(),
        VAR_STRING => {
            let cstr = if (*tv).vval.v_string.is_null() {
                ""
            } else {
                CStr::from_ptr((*tv).vval.v_string).to_str().unwrap_or("")
            };
            serde_json::to_string(&cstr).unwrap()
        }
        _ => "null".to_string(),
    };
    CString::new(s).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn rs_json_decode(ptr: *const c_char, tv: *mut typval_T) -> c_int {
    if ptr.is_null() || tv.is_null() {
        return -1;
    }
    let cstr = CStr::from_ptr(ptr).to_str().unwrap_or("");
    match serde_json::from_str::<Value>(cstr) {
        Ok(Value::Number(n)) => {
            (*tv).v_type = VAR_NUMBER;
            (*tv).vval.v_number = n.as_i64().unwrap_or(0);
            0
        }
        Ok(Value::String(s)) => {
            (*tv).v_type = VAR_STRING;
            (*tv).vval.v_string = CString::new(s).unwrap().into_raw();
            0
        }
        _ => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rs_json_encode_nr_expr(n: c_longlong) -> *mut c_char {
    let s = serde_json::to_string(&n).unwrap();
    CString::new(s).unwrap().into_raw()
}
