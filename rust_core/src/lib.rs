#![allow(unsafe_op_in_unsafe_fn)]

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::sync::{Mutex, OnceLock};

#[repr(C)]
#[derive(Clone, Copy)]
pub enum Vartype {
    VAR_UNKNOWN = 0,
    VAR_ANY,
    VAR_VOID,
    VAR_BOOL,
    VAR_SPECIAL,
    VAR_NUMBER,
    VAR_FLOAT,
    VAR_STRING,
}

#[repr(C)]
pub union ValUnion {
    pub v_number: i64,
    pub v_string: *mut c_char,
}

#[repr(C)]
pub struct typval_T {
    pub v_type: Vartype,
    pub v_lock: c_char,
    pub vval: ValUnion,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i64),
    Str(String),
}

impl Value {
    pub fn as_number(&self) -> Result<i64, ()> {
        match self {
            Value::Number(n) => Ok(*n),
            Value::Str(s) => s.parse().map_err(|_| ()),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::Str(s) => s.clone(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
        }
    }
}

pub unsafe fn to_typval(val: Value, out: *mut typval_T) {
    match val {
        Value::Number(n) => {
            (*out).v_type = Vartype::VAR_NUMBER;
            (*out).v_lock = 0;
            (*out).vval.v_number = n;
        }
        Value::Str(s) => {
            (*out).v_type = Vartype::VAR_STRING;
            (*out).v_lock = 0;
            let cstr = CString::new(s).unwrap();
            (*out).vval.v_string = cstr.into_raw();
        }
    }
}

pub unsafe fn from_typval(tv: *const typval_T) -> Option<Value> {
    if tv.is_null() {
        return None;
    }
    match (*tv).v_type {
        Vartype::VAR_NUMBER => Some(Value::Number((*tv).vval.v_number)),
        Vartype::VAR_STRING => {
            if (*tv).vval.v_string.is_null() {
                Some(Value::Str(String::new()))
            } else {
                let cstr = CStr::from_ptr((*tv).vval.v_string);
                cstr.to_str().ok().map(|s| Value::Str(s.to_string()))
            }
        }
        _ => None,
    }
}

#[no_mangle]
pub unsafe extern "C" fn tv_number(n: i64, out: *mut typval_T) {
    if !out.is_null() {
        to_typval(Value::Number(n), out);
    }
}

#[no_mangle]
pub unsafe extern "C" fn tv_string(s: *const c_char, out: *mut typval_T) {
    if out.is_null() || s.is_null() {
        return;
    }
    let val = CStr::from_ptr(s).to_string_lossy().into_owned();
    to_typval(Value::Str(val), out);
}

#[no_mangle]
pub unsafe extern "C" fn tv_free(tv: *mut typval_T) {
    if tv.is_null() {
        return;
    }
    if let Vartype::VAR_STRING = (*tv).v_type {
        if !(*tv).vval.v_string.is_null() {
            let _ = CString::from_raw((*tv).vval.v_string);
        }
    }
    (*tv).v_type = Vartype::VAR_UNKNOWN;
}

static ALLOCATIONS: OnceLock<Mutex<HashMap<usize, Vec<u8>>>> = OnceLock::new();

fn allocations() -> &'static Mutex<HashMap<usize, Vec<u8>>> {
    ALLOCATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[no_mangle]
pub extern "C" fn vim_alloc_rs(size: usize) -> *mut c_void {
    let mut buf: Vec<u8> = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    unsafe { buf.set_len(size); }
    allocations().lock().unwrap().insert(ptr as usize, buf);
    ptr as *mut c_void
}

#[no_mangle]
pub extern "C" fn alloc_clear_rs(size: usize) -> *mut c_void {
    let mut buf = vec![0u8; size];
    let ptr = buf.as_mut_ptr();
    allocations().lock().unwrap().insert(ptr as usize, buf);
    ptr as *mut c_void
}

#[no_mangle]
pub extern "C" fn vim_free_rs(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    allocations().lock().unwrap().remove(&(ptr as usize));
}

#[no_mangle]
pub extern "C" fn mem_realloc_rs(ptr: *mut c_void, size: usize) -> *mut c_void {
    if ptr.is_null() {
        return vim_alloc_rs(size);
    }
    let mut map = allocations().lock().unwrap();
    if let Some(old) = map.remove(&(ptr as usize)) {
        let mut new_buf = old;
        new_buf.resize(size, 0);
        let new_ptr = new_buf.as_mut_ptr();
        map.insert(new_ptr as usize, new_buf);
        new_ptr as *mut c_void
    } else {
        vim_alloc_rs(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_number() {
        let mut tv = typval_T { v_type: Vartype::VAR_UNKNOWN, v_lock: 0, vval: ValUnion { v_number: 0 } };
        unsafe {
            to_typval(Value::Number(42), &mut tv);
            assert_eq!(from_typval(&tv as *const typval_T), Some(Value::Number(42)));
        }
    }

    #[test]
    fn roundtrip_string() {
        let mut tv = typval_T { v_type: Vartype::VAR_UNKNOWN, v_lock: 0, vval: ValUnion { v_string: std::ptr::null_mut() } };
        unsafe {
            to_typval(Value::Str("hi".into()), &mut tv);
            assert_eq!(from_typval(&tv as *const typval_T), Some(Value::Str("hi".into())));
            tv_free(&mut tv);
        }
    }

    #[test]
    fn alloc_and_free() {
        let p = vim_alloc_rs(10);
        assert!(!p.is_null());
        vim_free_rs(p);
    }
}
