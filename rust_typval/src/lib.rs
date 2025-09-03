use libc::{c_char, c_long, c_uchar};

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

/// Allocate a zeroed typval_T.
#[no_mangle]
pub unsafe extern "C" fn alloc_tv() -> *mut typval_T {
    let tv: *mut typval_T = libc::calloc(1, std::mem::size_of::<typval_T>()) as *mut typval_T;
    tv
}

/// Free a typval previously allocated with `alloc_tv`.
#[no_mangle]
pub unsafe extern "C" fn free_tv(tv: *mut typval_T) {
    if tv.is_null() {
        return;
    }
    if (*tv).v_type == vartype_T::VAR_STRING {
        libc::free((*tv).vval.v_string as *mut _);
    }
    libc::free(tv as *mut _);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn alloc_and_free_string_tv() {
        unsafe {
            let tv = alloc_tv();
            (*tv).v_type = vartype_T::VAR_STRING;
            let s = CString::new("hello").unwrap();
            (*tv).vval.v_string = libc::strdup(s.as_ptr()) as *mut c_uchar;
            free_tv(tv);
        }
    }
}
