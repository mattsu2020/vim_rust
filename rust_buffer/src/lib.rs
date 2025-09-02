use std::collections::HashSet;
use std::os::raw::{c_int, c_void};
use std::sync::{Mutex, OnceLock};

// Keep track of allocated buffers so they can be cleaned up when buf_freeall is
// called from the C side.  The actual freeing of the memory is still performed
// in Vim's C code (free_buffer()), but tracking allocations here makes the
// behaviour visible and testable from Rust.
static BUFFERS: OnceLock<Mutex<HashSet<usize>>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn buf_alloc(size: usize) -> *mut c_void {
    let ptr = unsafe { libc::calloc(1, size) };
    if !ptr.is_null() {
        BUFFERS
            .get_or_init(|| Mutex::new(HashSet::new()))
            .lock()
            .unwrap()
            .insert(ptr as usize);
    }
    ptr
}

#[no_mangle]
pub extern "C" fn buf_freeall(buf: *mut c_void, _flags: c_int) {
    if buf.is_null() {
        return;
    }
    if let Some(m) = BUFFERS.get() {
        let mut buffers = m.lock().unwrap();
        buffers.remove(&(buf as usize));
    }
    // The buffer struct itself is freed in Vim's C code; we only keep track
    // of allocations here and clear our bookkeeping on free.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_free() {
        let p = buf_alloc(16);
        assert!(!p.is_null());
        // Ensure that the allocation is tracked.
        let set = BUFFERS.get().unwrap();
        assert!(set.lock().unwrap().contains(&(p as usize)));
        buf_freeall(p, 0);
        // After freeing the entry should have been removed from the tracker.
        assert!(!set.lock().unwrap().contains(&(p as usize)));
        unsafe { libc::free(p) };
    }
}
