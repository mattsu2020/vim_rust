use std::collections::HashMap;
use std::os::raw::{c_int, c_void};
use std::sync::{Mutex, OnceLock};

// Lazily initialized global map of allocations so that memory obtained in
// Rust can later be released from C code.
static ALLOCATIONS: OnceLock<Mutex<HashMap<usize, Vec<u8>>>> = OnceLock::new();

fn allocations() -> &'static Mutex<HashMap<usize, Vec<u8>>> {
    ALLOCATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn rust_alloc(size: usize) -> *mut c_void {
    let mut buf: Vec<u8> = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    // Safety: we reserve `size` bytes and assume caller initializes it.
    unsafe { buf.set_len(size); }
    allocations().lock().unwrap().insert(ptr as usize, buf);
    ptr as *mut c_void
}

fn rust_alloc_clear(size: usize) -> *mut c_void {
    let mut buf = vec![0u8; size];
    let ptr = buf.as_mut_ptr();
    allocations().lock().unwrap().insert(ptr as usize, buf);
    ptr as *mut c_void
}

fn rust_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    allocations().lock().unwrap().remove(&(ptr as usize));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn alloc(size: usize) -> *mut c_void {
    rust_alloc(size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn alloc_id(size: usize, _id: c_int) -> *mut c_void {
    rust_alloc(size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn alloc_clear(size: usize) -> *mut c_void {
    rust_alloc_clear(size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn alloc_clear_id(size: usize, _id: c_int) -> *mut c_void {
    rust_alloc_clear(size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lalloc(size: usize, _message: c_int) -> *mut c_void {
    rust_alloc(size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lalloc_clear(size: usize, _message: c_int) -> *mut c_void {
    rust_alloc_clear(size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lalloc_id(size: usize, message: c_int, _id: c_int) -> *mut c_void {
    unsafe { lalloc(size, message) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mem_realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    if ptr.is_null() {
        return rust_alloc(size);
    }
    let mut map = allocations().lock().unwrap();
    if let Some(old) = map.remove(&(ptr as usize)) {
        let copy_len = std::cmp::min(old.len(), size);
        let mut new_buf = Vec::with_capacity(size);
        new_buf.extend_from_slice(&old[..copy_len]);
        new_buf.resize(size, 0);
        let new_ptr = new_buf.as_mut_ptr();
        map.insert(new_ptr as usize, new_buf);
        new_ptr as *mut c_void
    } else {
        rust_alloc(size)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_free(ptr: *mut c_void) {
    rust_free(ptr);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_free() {
        let p = unsafe { alloc(10) };
        assert!(!p.is_null());
        unsafe { vim_free(p) };
    }

    #[test]
    fn realloc_grows() {
        let p = unsafe { alloc(4) };
        assert!(!p.is_null());
        let p2 = unsafe { mem_realloc(p, 8) };
        assert!(!p2.is_null());
        unsafe { vim_free(p2) };
    }
}
