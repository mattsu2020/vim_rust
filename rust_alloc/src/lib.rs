use libc::{c_void, size_t};
use std::ptr;

/// Allocate `size` bytes of memory using libc::malloc.
/// Returns a null pointer when allocation fails or size is zero.
#[no_mangle]
pub unsafe extern "C" fn rust_alloc(size: size_t) -> *mut c_void {
    if size == 0 {
        return ptr::null_mut();
    }
    let p = libc::malloc(size);
    if p.is_null() {
        ptr::null_mut()
    } else {
        p
    }
}

/// Allocate zero-initialized memory using libc::calloc.
#[no_mangle]
pub unsafe extern "C" fn rust_alloc_clear(size: size_t) -> *mut c_void {
    if size == 0 {
        return ptr::null_mut();
    }
    let p = libc::calloc(1, size);
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
        libc::free(ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::slice;

    #[test]
    fn allocate_and_free() {
        unsafe {
            let p = rust_alloc(8);
            assert!(!p.is_null());
            rust_free(p);
        }
    }

    #[test]
    fn allocate_clear_returns_zeroed() {
        unsafe {
            let p = rust_alloc_clear(4);
            assert!(!p.is_null());
            let slice = slice::from_raw_parts(p as *const u8, 4);
            assert_eq!(slice, &[0u8; 4]);
            rust_free(p);
        }
    }

    #[test]
    fn zero_size_returns_null() {
        unsafe {
            assert!(rust_alloc(0).is_null());
            assert!(rust_alloc_clear(0).is_null());
        }
    }
}
