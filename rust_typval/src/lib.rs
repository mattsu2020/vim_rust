use libc::{c_char, c_long, c_uchar};
use std::ffi::{CStr, CString};

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum vartype_T {
    VAR_UNKNOWN = 0,
    VAR_NUMBER = 1,
    VAR_STRING = 2,
}

#[repr(C)]
pub union typval_vval {
    pub v_number: c_long,
    pub v_string: *mut c_uchar,
}

#[repr(C)]
pub struct typval_T {
    pub v_type: vartype_T,
    pub v_lock: c_char,
    pub vval: typval_vval,
}

/// Safe Rust representation of a Vim typval.
#[derive(Clone, Debug, PartialEq)]
pub enum TypVal {
    Number(i64),
    String(String),
}

impl From<&typval_T> for TypVal {
    #[allow(clippy::unnecessary_cast)]
    fn from(tv: &typval_T) -> Self {
        unsafe {
            match tv.v_type {
                vartype_T::VAR_NUMBER => TypVal::Number(tv.vval.v_number as i64),
                vartype_T::VAR_STRING => {
                    let ptr = tv.vval.v_string;
                    let s = if ptr.is_null() {
                        String::new()
                    } else {
                        CStr::from_ptr(ptr as *const c_char)
                            .to_string_lossy()
                            .into_owned()
                    };
                    TypVal::String(s)
                }
                _ => TypVal::Number(0),
            }
        }
    }
}

impl From<TypVal> for typval_T {
    fn from(val: TypVal) -> Self {
        match val {
            TypVal::Number(n) => typval_T {
                v_type: vartype_T::VAR_NUMBER,
                v_lock: 0,
                vval: typval_vval {
                    v_number: n as c_long,
                },
            },
            TypVal::String(s) => {
                let c = CString::new(s).unwrap();
                let ptr = c.into_raw() as *mut c_uchar;
                typval_T {
                    v_type: vartype_T::VAR_STRING,
                    v_lock: 0,
                    vval: typval_vval { v_string: ptr },
                }
            }
        }
    }
}

/// Allocate a zeroed `typval_T`.
#[no_mangle]
pub extern "C" fn alloc_tv() -> *mut typval_T {
    Box::into_raw(Box::new(typval_T {
        v_type: vartype_T::VAR_UNKNOWN,
        v_lock: 0,
        vval: typval_vval { v_number: 0 },
    }))
}

/// Free a typval previously allocated with `alloc_tv` or converted from `TypVal`.
///
/// # Safety
/// `tv` must point to a valid `typval_T` that was allocated by `alloc_tv` or
/// created from a `TypVal`.  After calling this function the pointer must not
/// be used again.
#[no_mangle]
pub unsafe extern "C" fn free_tv(tv: *mut typval_T) {
    if tv.is_null() {
        return;
    }
    if (*tv).v_type == vartype_T::VAR_STRING && !(*tv).vval.v_string.is_null() {
        drop(CString::from_raw((*tv).vval.v_string as *mut c_char));
    }
    drop(Box::from_raw(tv));
}
