use std::os::raw::{c_void, c_int};

#[no_mangle]
pub extern "C" fn buf_alloc(size: usize) -> *mut c_void {
    unsafe { libc::calloc(1, size) }
}

#[no_mangle]
pub extern "C" fn buf_freeall(_buf: *mut c_void, _flags: c_int) {
    // Actual cleanup is performed on the C side.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_free() {
        let p = buf_alloc(16);
        assert!(!p.is_null());
        buf_freeall(p, 0);
        unsafe { libc::free(p) };
    }
}
