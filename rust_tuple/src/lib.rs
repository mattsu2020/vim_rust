use std::os::raw::c_long;

#[repr(C)]
pub struct rs_tuple {
    items: Vec<i64>,
}

#[no_mangle]
pub extern "C" fn rs_tuple_new(len: usize) -> *mut rs_tuple {
    let tuple = rs_tuple { items: vec![0; len] };
    Box::into_raw(Box::new(tuple))
}

#[no_mangle]
pub extern "C" fn rs_tuple_set(tuple: *mut rs_tuple, idx: usize, value: c_long) -> bool {
    if tuple.is_null() {
        return false;
    }
    let t = unsafe { &mut *tuple };
    if let Some(slot) = t.items.get_mut(idx) {
        *slot = value as i64;
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn rs_tuple_get(tuple: *const rs_tuple, idx: usize, value: *mut c_long) -> bool {
    if tuple.is_null() || value.is_null() {
        return false;
    }
    let t = unsafe { &*tuple };
    if let Some(&v) = t.items.get(idx) {
        unsafe { *value = v as c_long; }
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn rs_tuple_len(tuple: *const rs_tuple) -> usize {
    if tuple.is_null() {
        0
    } else {
        unsafe { (*tuple).items.len() }
    }
}

#[no_mangle]
pub extern "C" fn rs_tuple_free(tuple: *mut rs_tuple) {
    if tuple.is_null() {
        return;
    }
    unsafe { drop(Box::from_raw(tuple)); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_set_get() {
        let t = rs_tuple_new(3);
        assert_eq!(unsafe { rs_tuple_len(t) }, 3);
        assert!(unsafe { rs_tuple_set(t, 1, 42) });
        let mut val: c_long = 0;
        assert!(unsafe { rs_tuple_get(t, 1, &mut val) });
        assert_eq!(val, 42);
        unsafe { rs_tuple_free(t) };
    }
}
