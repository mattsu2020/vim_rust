use std::os::raw::c_void;
use std::ptr;

#[repr(C)]
pub struct VimDict {
    _private: *mut c_void,
}

#[no_mangle]
pub extern "C" fn rust_dict_new() -> *mut VimDict {
    Box::into_raw(Box::new(VimDict { _private: ptr::null_mut() }))
}

#[no_mangle]
pub extern "C" fn rust_dict_free(d: *mut VimDict) {
    if !d.is_null() {
        unsafe {
            drop(Box::from_raw(d));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_free() {
        let d = rust_dict_new();
        assert!(!d.is_null());
        rust_dict_free(d);
    }
}
