use libc::{c_char, c_int, c_void};
use std::ptr::{null_mut};
use std::slice;

#[repr(C)]
pub struct garray_T {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

#[repr(C)]
pub struct blob_T {
    bv_ga: garray_T,
    bv_refcount: c_int,
    bv_lock: c_char,
}

#[repr(C)]
pub union vval_u {
    v_blob: *mut blob_T,
    v_number: i64,
}

#[repr(C)]
pub struct typval_T {
    v_type: c_int,
    v_lock: c_char,
    vval: vval_u,
}

const VAR_BLOB: c_int = 8; // from enum vartype_T in structs.h
const OK: c_int = 1;
const FAIL: c_int = 0;

#[no_mangle]
pub extern "C" fn blob_alloc() -> *mut blob_T {
    let blob = Box::new(blob_T {
        bv_ga: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 1,
            ga_growsize: 100,
            ga_data: null_mut(),
        },
        bv_refcount: 0,
        bv_lock: 0,
    });
    Box::into_raw(blob)
}

#[no_mangle]
pub extern "C" fn rettv_blob_set(rettv: *mut typval_T, b: *mut blob_T) {
    unsafe {
        (*rettv).v_type = VAR_BLOB;
        (*rettv).vval.v_blob = b;
        if !b.is_null() {
            (*b).bv_refcount += 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn rettv_blob_alloc(rettv: *mut typval_T) -> c_int {
    let b = blob_alloc();
    if b.is_null() {
        return FAIL;
    }
    rettv_blob_set(rettv, b);
    OK
}

#[no_mangle]
pub extern "C" fn blob_copy(from: *mut blob_T, to: *mut typval_T) -> c_int {
    unsafe {
        (*to).v_type = VAR_BLOB;
        (*to).v_lock = 0;
        if from.is_null() {
            (*to).vval.v_blob = null_mut();
            return OK;
        }
        if rettv_blob_alloc(to) == FAIL {
            return FAIL;
        }
        let len = (*from).bv_ga.ga_len;
        if len > 0 {
            let size = len as usize;
            let src = slice::from_raw_parts((*from).bv_ga.ga_data as *const u8, size);
            let mut vec = Vec::with_capacity(size);
            vec.extend_from_slice(src);
            let data = Box::into_raw(vec.into_boxed_slice()) as *mut u8;
            let to_blob = (*to).vval.v_blob;
            if !to_blob.is_null() {
                (*to_blob).bv_ga.ga_data = data as *mut c_void;
                (*to_blob).bv_ga.ga_len = len;
                (*to_blob).bv_ga.ga_maxlen = len;
            }
        }
    }
    OK
}
