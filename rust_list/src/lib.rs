use std::os::raw::c_void;
use std::ptr;

#[repr(C)]
pub struct VimList {
    _private: *mut c_void,
}

#[no_mangle]
pub extern "C" fn rust_list_new() -> *mut VimList {
    Box::into_raw(Box::new(VimList { _private: ptr::null_mut() }))
}

#[no_mangle]
pub extern "C" fn rust_list_free(l: *mut VimList) {
    if !l.is_null() {
        unsafe {
            drop(Box::from_raw(l));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_free() {
        let l = rust_list_new();
        assert!(!l.is_null());
        rust_list_free(l);
    }
}
