use libc::{c_int, c_long};

#[repr(C)]
pub struct blob_T {
    data: Vec<u8>,
    refcount: i32,
}

#[no_mangle]
pub extern "C" fn blob_alloc() -> *mut blob_T {
    Box::into_raw(Box::new(blob_T { data: Vec::new(), refcount: 1 }))
}

#[no_mangle]
pub extern "C" fn blob_free(b: *mut blob_T) {
    if b.is_null() { return; }
    unsafe { drop(Box::from_raw(b)); }
}

#[no_mangle]
pub extern "C" fn blob_unref(b: *mut blob_T) {
    if b.is_null() { return; }
    unsafe {
        (*b).refcount -= 1;
        if (*b).refcount <= 0 {
            drop(Box::from_raw(b));
        }
    }
}

#[no_mangle]
pub extern "C" fn blob_len(b: *const blob_T) -> c_long {
    if b.is_null() { return 0; }
    let br = unsafe { &*b };
    br.data.len() as c_long
}

#[no_mangle]
pub extern "C" fn blob_get(b: *const blob_T, idx: c_int) -> c_int {
    let br = unsafe { &*b };
    br.data[idx as usize] as c_int
}

#[no_mangle]
pub extern "C" fn blob_set(b: *mut blob_T, idx: c_int, byte: c_int) {
    let data = unsafe { &mut (*b).data };
    let idx = idx as usize;
    if idx >= data.len() {
        data.resize(idx + 1, 0);
    }
    data[idx] = byte as u8;
}

#[no_mangle]
pub extern "C" fn blob_set_append(b: *mut blob_T, idx: c_int, byte: c_int) {
    blob_set(b, idx, byte);
}

#[no_mangle]
pub extern "C" fn blob_equal(b1: *const blob_T, b2: *const blob_T) -> c_int {
    if b1 == b2 { return 1; }
    let v1 = unsafe { &*b1 };
    let v2 = unsafe { &*b2 };
    (v1.data == v2.data) as c_int
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_blob() {
        unsafe {
            let b = blob_alloc();
            blob_set(b, 0, 1);
            blob_set(b, 1, 2);
            assert_eq!(blob_len(b), 2);
            assert_eq!(blob_get(b, 1), 2);
            blob_unref(b);
        }
    }

    #[test]
    fn equality() {
        unsafe {
            let b1 = blob_alloc();
            let b2 = blob_alloc();
            blob_set(b1, 0, 42);
            blob_set(b2, 0, 42);
            assert_eq!(blob_equal(b1, b2), 1);
            blob_unref(b1);
            blob_unref(b2);
        }
    }
}
