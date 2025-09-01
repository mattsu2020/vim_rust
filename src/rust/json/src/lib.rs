use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use serde_json::Value;

#[repr(C)]
pub struct garray_T {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

extern "C" {
    fn ga_concat_len(gap: *mut garray_T, s: *const u8, len: usize);
}

#[repr(C)]
pub union vval_T {
    v_number: i64,
    v_float: f64,
    v_string: *mut u8,
    v_list: *mut c_void,
    v_dict: *mut c_void,
}

#[repr(C)]
pub struct typval_T {
    v_type: c_int,
    v_lock: u8,
    vval: vval_T,
}

const VAR_BOOL: c_int = 3;
const VAR_SPECIAL: c_int = 4;
const VAR_NUMBER: c_int = 5;
const VAR_FLOAT: c_int = 6;
const VAR_STRING: c_int = 7;

const VVAL_FALSE: i64 = 0;
const VVAL_TRUE: i64 = 1;
const VVAL_NONE: i64 = 2;
const VVAL_NULL: i64 = 3;

unsafe fn tv_to_value(tv: *mut typval_T) -> Value {
    if tv.is_null() {
        return Value::Null;
    }
    match (*tv).v_type {
        VAR_NUMBER => Value::Number((*tv).vval.v_number.into()),
        VAR_BOOL => Value::Bool((*tv).vval.v_number == VVAL_TRUE),
        VAR_SPECIAL => match (*tv).vval.v_number {
            VVAL_TRUE => Value::Bool(true),
            VVAL_FALSE => Value::Bool(false),
            _ => Value::Null,
        },
        VAR_STRING => {
            let ptr = (*tv).vval.v_string;
            if ptr.is_null() {
                Value::String(String::new())
            } else {
                let cstr = CStr::from_ptr(ptr as *const c_char);
                Value::String(cstr.to_string_lossy().into_owned())
            }
        }
        _ => Value::Null,
    }
}

#[no_mangle]
pub unsafe extern "C" fn json_encode_gap(
    gap: *mut garray_T,
    val: *mut typval_T,
    _options: c_int,
) -> c_int {
    let value = tv_to_value(val);
    let json = match serde_json::to_string(&value) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    ga_concat_len(gap, json.as_ptr(), json.len());
    1
}

#[no_mangle]
pub unsafe extern "C" fn json_encode(
    val: *mut typval_T,
    _options: c_int,
) -> *mut u8 {
    let value = tv_to_value(val);
    let json = match serde_json::to_string(&value) {
        Ok(s) => s,
        Err(_) => String::new(),
    };
    CString::new(json).unwrap().into_raw() as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn json_encode_nr_expr(
    nr: c_int,
    val: *mut typval_T,
    _options: c_int,
) -> *mut u8 {
    let arr = Value::Array(vec![Value::Number(nr.into()), tv_to_value(val)]);
    let json = serde_json::to_string(&arr).unwrap_or_default();
    CString::new(json).unwrap().into_raw() as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn json_encode_lsp_msg(
    val: *mut typval_T,
) -> *mut u8 {
    let json_ptr = json_encode(val, 0);
    if json_ptr.is_null() {
        return std::ptr::null_mut();
    }
    let json_cstr = CStr::from_ptr(json_ptr as *const c_char);
    let json_bytes = json_cstr.to_bytes();
    let header = format!("Content-Length: {}\r\n\r\n", json_bytes.len());
    let mut combined = header.into_bytes();
    combined.extend_from_slice(json_bytes);
    libc::free(json_ptr as *mut c_void);
    CString::new(combined).unwrap().into_raw() as *mut u8
}

#[repr(C)]
pub struct js_read_T {
    js_buf: *mut u8,
    js_end: *mut u8,
    js_used: c_int,
    js_fill: Option<extern "C" fn(*mut js_read_T) -> c_int>,
    js_cookie: *mut c_void,
    js_cookie_arg: c_int,
}

unsafe fn value_to_tv(val: Value, tv: *mut typval_T) {
    match val {
        Value::Null => {
            (*tv).v_type = VAR_SPECIAL;
            (*tv).vval.v_number = VVAL_NULL;
        }
        Value::Bool(b) => {
            (*tv).v_type = VAR_BOOL;
            (*tv).vval.v_number = if b { VVAL_TRUE } else { VVAL_FALSE };
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                (*tv).v_type = VAR_NUMBER;
                (*tv).vval.v_number = i;
            } else if let Some(f) = n.as_f64() {
                (*tv).v_type = VAR_FLOAT;
                (*tv).vval.v_float = f;
            } else {
                (*tv).v_type = VAR_SPECIAL;
                (*tv).vval.v_number = VVAL_NULL;
            }
        }
        Value::String(s) => {
            (*tv).v_type = VAR_STRING;
            (*tv).vval.v_string = CString::new(s).unwrap().into_raw() as *mut u8;
        }
        _ => {
            (*tv).v_type = VAR_SPECIAL;
            (*tv).vval.v_number = VVAL_NULL;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn json_decode(
    reader: *mut js_read_T,
    res: *mut typval_T,
    _options: c_int,
) -> c_int {
    let cstr = CStr::from_ptr((*reader).js_buf as *const c_char);
    let txt = cstr.to_string_lossy();
    match serde_json::from_str::<Value>(&txt) {
        Ok(v) => {
            value_to_tv(v, res);
            (*reader).js_used = txt.len() as c_int;
            1
        }
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn json_find_end(
    reader: *mut js_read_T,
    _options: c_int,
) -> c_int {
    let cstr = CStr::from_ptr((*reader).js_buf as *const c_char);
    let txt = cstr.to_string_lossy();
    match serde_json::from_str::<Value>(&txt) {
        Ok(_) => {
            (*reader).js_used = txt.len() as c_int;
            1
        }
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn f_json_encode(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
) {
    (*rettv).vval.v_string = json_encode(argvars, 0);
    (*rettv).v_type = VAR_STRING;
}

#[no_mangle]
pub unsafe extern "C" fn f_json_decode(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
) {
    let mut reader = js_read_T {
        js_buf: (*argvars).vval.v_string,
        js_end: std::ptr::null_mut(),
        js_used: 0,
        js_fill: None,
        js_cookie: std::ptr::null_mut(),
        js_cookie_arg: 0,
    };
    json_decode(&mut reader, rettv, 0);
}

#[no_mangle]
pub unsafe extern "C" fn f_js_encode(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
) {
    f_json_encode(argvars, rettv);
}

#[no_mangle]
pub unsafe extern "C" fn f_js_decode(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
) {
    f_json_decode(argvars, rettv);
}
