use std::cell::RefCell;
use std::os::raw::c_int;
use std::sync::Arc;

/// Opaque Blob structure managed on the Rust side.
#[repr(C)]
pub struct Blob {
    data: RefCell<Vec<u8>>,
}

#[no_mangle]
pub extern "C" fn blob_alloc() -> *const Blob {
    Arc::into_raw(Arc::new(Blob {
        data: RefCell::new(Vec::new()),
    }))
}

#[no_mangle]
pub extern "C" fn blob_ref(b: *const Blob) -> *const Blob {
    if b.is_null() {
        return std::ptr::null();
    }
    unsafe {
        Arc::increment_strong_count(b);
    }
    b
}

#[no_mangle]
pub extern "C" fn blob_unref(b: *const Blob) {
    if b.is_null() {
        return;
    }
    unsafe {
        // Drop one strong reference.
        drop(Arc::from_raw(b));
    }
}

#[no_mangle]
pub extern "C" fn blob_len(b: *const Blob) -> usize {
    if b.is_null() {
        return 0;
    }
    unsafe { (&*b).data.borrow().len() }
}

#[no_mangle]
pub extern "C" fn blob_get(b: *const Blob, idx: usize) -> u8 {
    if b.is_null() {
        return 0;
    }
    unsafe {
        let data = (&*b).data.borrow();
        if idx >= data.len() {
            0
        } else {
            data[idx]
        }
    }
}

#[no_mangle]
pub extern "C" fn blob_set_append(b: *const Blob, idx: usize, byte: u8) {
    if b.is_null() {
        return;
    }
    unsafe {
        let mut data = (&*b).data.borrow_mut();
        if idx < data.len() {
            data[idx] = byte;
        } else if idx == data.len() {
            data.push(byte);
        }
    }
}

#[no_mangle]
pub extern "C" fn blob_equal(b1: *const Blob, b2: *const Blob) -> c_int {
    unsafe {
        let len1 = blob_len(b1);
        let len2 = blob_len(b2);
        if len1 == 0 && len2 == 0 {
            return 1;
        }
        if b1 == b2 {
            return 1;
        }
        if len1 != len2 {
            return 0;
        }
        let d1 = (&*b1).data.borrow();
        let d2 = (&*b2).data.borrow();
        (d1.as_slice() == d2.as_slice()) as c_int
    }
}

#[no_mangle]
pub extern "C" fn blob_clone(b: *const Blob) -> *const Blob {
    if b.is_null() {
        return std::ptr::null();
    }
    unsafe {
        let data = (&*b).data.borrow().clone();
        Arc::into_raw(Arc::new(Blob {
            data: RefCell::new(data),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_get() {
        let b = blob_alloc();
        assert_eq!(blob_len(b), 0);
        blob_set_append(b, 0, 10);
        blob_set_append(b, 1, 20);
        assert_eq!(blob_len(b), 2);
        assert_eq!(blob_get(b, 1), 20);
        blob_unref(b);
    }

    #[test]
    fn refcounting() {
        let b = blob_alloc();
        let b2 = blob_ref(b);
        blob_unref(b);
        blob_unref(b2);
    }

    #[test]
    fn equality() {
        let b1 = blob_alloc();
        blob_set_append(b1, 0, 1);
        blob_set_append(b1, 1, 2);
        let b2 = blob_clone(b1);
        assert_eq!(blob_equal(b1, b2), 1);
        blob_set_append(b2, 1, 3);
        assert_eq!(blob_equal(b1, b2), 0);
        blob_unref(b1);
        blob_unref(b2);
    }
}
