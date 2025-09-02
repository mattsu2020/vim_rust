use std::os::raw::{c_int, c_void};
use std::sync::{Mutex, OnceLock};

// Maintain a list of all allocated buffers so that they can be released safely
// from Rust.  The buffers are allocated using libc and can therefore be freed
// here as well, avoiding the need for C code to perform the deallocation.

#[repr(C)]
pub struct FileBuffer {
    _data: [u8; 0],
}

// Global storage of allocated buffer pointers.  A simple Vec is sufficient
// because we only ever store raw pointers and occasionally remove them.
static BUFFERS: OnceLock<Mutex<Vec<usize>>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn buf_alloc(size: usize) -> *mut FileBuffer {
    let ptr = unsafe { libc::calloc(1, size) } as *mut FileBuffer;
    if !ptr.is_null() {
        BUFFERS
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .unwrap()
            .push(ptr as *mut c_void as usize);
    }
    ptr
}

#[no_mangle]
pub extern "C" fn buf_free(buf: *mut FileBuffer) {
    if buf.is_null() {
        return;
    }
    if let Some(m) = BUFFERS.get() {
        let mut buffers = m.lock().unwrap();
        if let Some(pos) = buffers
            .iter()
            .position(|&p| p == buf as *mut c_void as usize)
        {
            buffers.remove(pos);
        }
    }
    unsafe { libc::free(buf as *mut c_void) };
}

#[no_mangle]
pub extern "C" fn buf_freeall(_buf: *mut FileBuffer, _flags: c_int) {
    if let Some(m) = BUFFERS.get() {
        let mut buffers = m.lock().unwrap();
        for ptr in buffers.drain(..) {
            unsafe { libc::free(ptr as *mut c_void) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::raw::c_void;

    #[test]
    fn alloc_and_free() {
        let p = buf_alloc(16);
        assert!(!p.is_null());
        let list = BUFFERS.get().unwrap();
        assert!(list.lock().unwrap().contains(&(p as *mut c_void as usize)));
        buf_free(p);
        assert!(!list.lock().unwrap().contains(&(p as *mut c_void as usize)));
    }

    #[test]
    fn multiple_allocations_and_free_all() {
        let p1 = buf_alloc(8);
        let p2 = buf_alloc(8);
        assert!(!p1.is_null() && !p2.is_null());
        let list = BUFFERS.get().unwrap();
        {
            let guard = list.lock().unwrap();
            assert!(guard.contains(&(p1 as *mut c_void as usize)));
            assert!(guard.contains(&(p2 as *mut c_void as usize)));
        }
        buf_freeall(std::ptr::null_mut(), 0);
        assert!(list.lock().unwrap().is_empty());
    }
}
