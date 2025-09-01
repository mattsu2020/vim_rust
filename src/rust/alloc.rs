use std::ffi::c_void;
use std::ptr;

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn calloc(nmemb: usize, size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

/// Allocate `size` bytes of memory using the C allocator.
/// Returns null on error or when `size` is zero.
#[no_mangle]
pub unsafe extern "C" fn rust_alloc(size: usize) -> *mut c_void {
    if size == 0 {
        return ptr::null_mut();
    }
    let p = malloc(size);
    if p.is_null() {
        ptr::null_mut()
    } else {
        p
    }
}

/// Allocate zero-initialized memory using the C allocator.
/// Returns null on error or when `size` is zero.
#[no_mangle]
pub unsafe extern "C" fn rust_alloc_clear(size: usize) -> *mut c_void {
    if size == 0 {
        return ptr::null_mut();
    }
    let p = calloc(1, size);
    if p.is_null() {
        ptr::null_mut()
    } else {
        p
    }
}

/// Free memory previously allocated with `rust_alloc` or `rust_alloc_clear`.
#[no_mangle]
pub unsafe extern "C" fn rust_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        free(ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn rust_alloc_and_c_free() {
        unsafe {
            let ptr = rust_alloc(64);
            assert!(!ptr.is_null());
            assert_eq!((ptr as usize) % mem::align_of::<usize>(), 0);
            free(ptr);
        }
    }

    #[test]
    fn c_alloc_and_rust_free() {
        unsafe {
            let ptr = malloc(64);
            assert!(!ptr.is_null());
            rust_free(ptr);
        }
    }
}
