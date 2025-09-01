use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

#[repr(C)]
#[derive(Debug)]
pub enum TypVal {
    Number(i64),
    Float(f64),
    String(*mut c_char),
}

impl Clone for TypVal {
    fn clone(&self) -> Self {
        match *self {
            TypVal::Number(n) => TypVal::Number(n),
            TypVal::Float(f) => TypVal::Float(f),
            TypVal::String(ptr) => {
                if ptr.is_null() {
                    TypVal::String(std::ptr::null_mut())
                } else {
                    unsafe {
                        let cstr = CStr::from_ptr(ptr);
                        let owned = CString::new(cstr.to_bytes()).unwrap();
                        TypVal::String(owned.into_raw())
                    }
                }
            }
        }
    }
}

impl PartialEq for TypVal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypVal::Number(a), TypVal::Number(b)) => a == b,
            (TypVal::Float(a), TypVal::Float(b)) => a == b,
            (TypVal::String(a), TypVal::String(b)) => unsafe {
                match (*a, *b) {
                    (pa, pb) if pa.is_null() || pb.is_null() => pa == pb,
                    (pa, pb) => CStr::from_ptr(pa) == CStr::from_ptr(pb),
                }
            },
            _ => false,
        }
    }
}

#[no_mangle]
pub extern "C" fn typval_new_number(n: i64) -> *mut TypVal {
    Box::into_raw(Box::new(TypVal::Number(n)))
}

#[no_mangle]
pub extern "C" fn typval_new_float(f: f64) -> *mut TypVal {
    Box::into_raw(Box::new(TypVal::Float(f)))
}

#[no_mangle]
pub extern "C" fn typval_new_string(s: *const c_char) -> *mut TypVal {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let cstr = CStr::from_ptr(s);
        match CString::new(cstr.to_bytes()) {
            Ok(owned) => {
                let ptr = owned.into_raw();
                Box::into_raw(Box::new(TypVal::String(ptr)))
            }
            Err(_) => std::ptr::null_mut(),
        }
    }
}

#[no_mangle]
pub extern "C" fn typval_copy(src: *const TypVal) -> *mut TypVal {
    if src.is_null() {
        return std::ptr::null_mut();
    }
    let val = unsafe { (*src).clone() };
    Box::into_raw(Box::new(val))
}

#[no_mangle]
pub extern "C" fn typval_compare(a: *const TypVal, b: *const TypVal) -> c_int {
    if a.is_null() || b.is_null() {
        return -1;
    }
    let eq = unsafe { (*a) == (*b) };
    if eq { 0 } else { 1 }
}

#[no_mangle]
pub extern "C" fn typval_free(val: *mut TypVal) {
    if val.is_null() {
        return;
    }
    unsafe {
        match *val {
            TypVal::String(ptr) if !ptr.is_null() => {
                let _ = CString::from_raw(ptr);
            }
            _ => {}
        }
        drop(Box::from_raw(val));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn number_roundtrip() {
        let tv = typval_new_number(42);
        let copy = typval_copy(tv);
        assert_eq!(typval_compare(tv, copy), 0);
        typval_free(tv);
        typval_free(copy);
    }

    #[test]
    fn string_roundtrip() {
        let s = CString::new("hello").unwrap();
        let tv = typval_new_string(s.as_ptr());
        let copy = typval_copy(tv);
        assert_eq!(typval_compare(tv, copy), 0);
        typval_free(tv);
        typval_free(copy);
    }
}
