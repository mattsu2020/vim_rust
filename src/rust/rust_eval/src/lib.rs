use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uchar};

#[repr(C)]
#[derive(Clone, Copy)]
pub enum vartype_T {
    VAR_UNKNOWN = 0,
    VAR_NUMBER = 1,
    VAR_STRING = 2,
    // Other variants omitted for brevity
}

#[repr(C)]
pub union typval_vval {
    pub v_number: libc::c_long, // corresponds to varnumber_T
    pub v_string: *mut c_uchar,
}

#[repr(C)]
pub struct typval_T {
    pub v_type: vartype_T,
    pub v_lock: c_char,
    pub vval: typval_vval,
}

/// High-level Rust representation of Vim's typval_T.
#[derive(Debug, Clone)]
pub enum TypVal {
    Number(i64),
    String(String),
    /// Placeholder for unsupported types.
    Other,
}

impl From<typval_T> for TypVal {
    fn from(tv: typval_T) -> Self {
        unsafe {
            match tv.v_type {
                vartype_T::VAR_NUMBER => TypVal::Number(tv.vval.v_number as i64),
                vartype_T::VAR_STRING => {
                    if tv.vval.v_string.is_null() {
                        TypVal::String(String::new())
                    } else {
                        let c_str = CStr::from_ptr(tv.vval.v_string as *const c_char);
                        TypVal::String(c_str.to_string_lossy().into_owned())
                    }
                }
                _ => TypVal::Other,
            }
        }
    }
}

impl From<TypVal> for typval_T {
    fn from(tv: TypVal) -> Self {
        match tv {
            TypVal::Number(n) => typval_T {
                v_type: vartype_T::VAR_NUMBER,
                v_lock: 0,
                vval: typval_vval { v_number: n as libc::c_long },
            },
            TypVal::String(s) => {
                let c_string = CString::new(s).unwrap();
                typval_T {
                    v_type: vartype_T::VAR_STRING,
                    v_lock: 0,
                    vval: typval_vval {
                        v_string: c_string.into_raw() as *mut c_uchar,
                    },
                }
            }
            TypVal::Other => typval_T {
                v_type: vartype_T::VAR_UNKNOWN,
                v_lock: 0,
                vval: typval_vval { v_number: 0 },
            },
        }
    }
}
