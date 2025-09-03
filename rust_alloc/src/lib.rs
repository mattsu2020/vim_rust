use std::collections::HashMap;
use std::os::raw::c_void;
use std::sync::{Mutex, OnceLock};

// Lazily initialized global map of allocations so that memory obtained in
// Rust can later be released from C code.
static ALLOCATIONS: OnceLock<Mutex<HashMap<usize, Vec<u8>>>> = OnceLock::new();

fn allocations() -> &'static Mutex<HashMap<usize, Vec<u8>>> {
    ALLOCATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn rust_alloc_impl(size: usize) -> *mut c_void {
    let mut buf: Vec<u8> = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    // Safety: we reserve `size` bytes and assume caller initializes it.
    unsafe { buf.set_len(size); }
    allocations().lock().unwrap().insert(ptr as usize, buf);
    ptr as *mut c_void
}

fn rust_alloc_clear_impl(size: usize) -> *mut c_void {
    let mut buf = vec![0u8; size];
    let ptr = buf.as_mut_ptr();
    allocations().lock().unwrap().insert(ptr as usize, buf);
    ptr as *mut c_void
}

fn rust_free_impl(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    allocations().lock().unwrap().remove(&(ptr as usize));
}

// Note: The C side provides alloc()/lalloc()/vim_free() implementations.
// Export only rust_* symbols to avoid duplicate definitions at link time.

// Export the symbol names expected by the C side
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_alloc(size: usize) -> *mut c_void {
    rust_alloc_impl(size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_alloc_clear(size: usize) -> *mut c_void {
    rust_alloc_clear_impl(size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_free(ptr: *mut c_void) {
    rust_free_impl(ptr)
}

// Safe reallocation that preserves as much as possible of the old contents.
// This function is aware of buffers originally allocated through rust_alloc
// and uses the tracked size to avoid reading/writing out of bounds.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_mem_realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    if ptr.is_null() {
        return rust_alloc_impl(size);
    }
    let mut map = allocations().lock().unwrap();
    if let Some(old) = map.remove(&(ptr as usize)) {
        let copy_len = core::cmp::min(old.len(), size);
        let mut new_buf = Vec::with_capacity(size);
        new_buf.extend_from_slice(&old[..copy_len]);
        new_buf.resize(size, 0);
        let new_ptr = new_buf.as_mut_ptr();
        map.insert(new_ptr as usize, new_buf);
        new_ptr as *mut c_void
    } else {
        // Pointer not tracked (likely not from rust_alloc). Allocate fresh.
        // Content cannot be safely preserved.
        rust_alloc_impl(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_free() {
        let p = unsafe { rust_alloc(10) };
        assert!(!p.is_null());
        unsafe { rust_free(p) };
    }

    #[test]
    fn realloc_grows() {
        let p = unsafe { rust_alloc(4) };
        assert!(!p.is_null());
        let p2 = unsafe { rust_mem_realloc(p, 8) };
        assert!(!p2.is_null());
        unsafe { rust_free(p2) };
    }
}
