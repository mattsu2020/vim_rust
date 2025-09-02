use std::os::raw::{c_int, c_void};

extern "C" {
    static mut alloc_fail_id: c_int;
    fn alloc_does_fail(size: usize) -> c_int;
}

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    unsafe { libc::malloc(size) }
}

#[no_mangle]
pub extern "C" fn alloc_id(size: usize, id: c_int) -> *mut c_void {
    unsafe {
        if alloc_fail_id == id && alloc_does_fail(size) != 0 {
            return std::ptr::null_mut();
        }
    }
    alloc(size)
}

#[no_mangle]
pub extern "C" fn alloc_clear(size: usize) -> *mut c_void {
    unsafe { libc::calloc(1, size) }
}

#[no_mangle]
pub extern "C" fn alloc_clear_id(size: usize, id: c_int) -> *mut c_void {
    unsafe {
        if alloc_fail_id == id && alloc_does_fail(size) != 0 {
            return std::ptr::null_mut();
        }
    }
    alloc_clear(size)
}

#[no_mangle]
pub extern "C" fn lalloc_clear(size: usize, _message: c_int) -> *mut c_void {
    alloc_clear(size)
}

#[no_mangle]
pub extern "C" fn lalloc(size: usize, _message: c_int) -> *mut c_void {
    alloc(size)
}

#[no_mangle]
pub extern "C" fn lalloc_id(size: usize, message: c_int, id: c_int) -> *mut c_void {
    unsafe {
        if alloc_fail_id == id && alloc_does_fail(size) != 0 {
            return std::ptr::null_mut();
        }
    }
    lalloc(size, message)
}

#[no_mangle]
pub extern "C" fn mem_realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    unsafe { libc::realloc(ptr, size) }
}

#[no_mangle]
pub extern "C" fn vim_free(x: *mut c_void) {
    unsafe { libc::free(x) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_free() {
        unsafe {
            let p = alloc(16);
            assert!(!p.is_null());
            vim_free(p);
        }
    }

    #[test]
    fn realloc_works() {
        unsafe {
            let mut p = alloc(8);
            assert!(!p.is_null());
            p = mem_realloc(p, 16);
            assert!(!p.is_null());
            vim_free(p);
        }
    }
}
