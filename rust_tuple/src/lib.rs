use libc::{c_int, c_long};

#[repr(C)]
pub struct garray_T {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut libc::c_void,
}

#[repr(C)]
pub struct tuple_T {
    tv_items: garray_T,
    tv_type: *mut libc::c_void,
    tv_copytuple: *mut libc::c_void,
    tv_used_next: *mut tuple_T,
    tv_used_prev: *mut tuple_T,
    tv_refcount: c_int,
    tv_copyID: c_int,
    tv_lock: libc::c_char,
}

/// Return the number of items in a tuple.
#[no_mangle]
pub unsafe extern "C" fn tuple_len(tuple: *const tuple_T) -> c_long {
    if tuple.is_null() {
        0
    } else {
        (*tuple).tv_items.ga_len as c_long
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len_of_null_tuple_is_zero() {
        let len = unsafe { tuple_len(std::ptr::null()) };
        assert_eq!(len, 0);
    }
}
