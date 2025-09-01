use std::collections::HashMap;
use std::os::raw::c_void;
use std::sync::{Mutex, OnceLock};

// Lazily initialized global map of allocations so that memory obtained in
// Rust can later be released from C code.
static ALLOCATIONS: OnceLock<Mutex<HashMap<usize, Vec<u8>>>> = OnceLock::new();

fn allocations() -> &'static Mutex<HashMap<usize, Vec<u8>>> {
    ALLOCATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[no_mangle]
pub extern "C" fn rust_alloc(size: usize) -> *mut c_void {
    let mut buf: Vec<u8> = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    // Safety: we reserve `size` bytes and assume caller initializes it.
    unsafe { buf.set_len(size); }
    allocations().lock().unwrap().insert(ptr as usize, buf);
    ptr as *mut c_void
}

#[no_mangle]
pub extern "C" fn rust_alloc_clear(size: usize) -> *mut c_void {
    let mut buf = vec![0u8; size];
    let ptr = buf.as_mut_ptr();
    allocations().lock().unwrap().insert(ptr as usize, buf);
    ptr as *mut c_void
}

#[no_mangle]
pub extern "C" fn rust_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    allocations().lock().unwrap().remove(&(ptr as usize));
}
