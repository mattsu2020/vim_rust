use std::os::raw::{c_int, c_long, c_void};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Mutex, OnceLock,
};

// Maintain a list of all allocated buffers so that they can be released safely
// from Rust.  The buffers are allocated using libc and can therefore be freed
// here as well, avoiding the need for C code to perform the deallocation.

#[repr(C)]
pub struct FileBuffer {
    _data: [u8; 0],
}

#[repr(transparent)]
pub struct OwnedFileBuffer(*mut FileBuffer);

impl OwnedFileBuffer {
    pub fn as_ptr(&self) -> *mut FileBuffer {
        self.0
    }
}

impl Drop for OwnedFileBuffer {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        if let Some(m) = BUFFERS.get() {
            let mut buffers = m.lock().unwrap();
            if let Some(pos) = buffers.iter().position(|&p| p == self.0 as usize) {
                buffers.remove(pos);
            }
        }
        unsafe { libc::free(self.0 as *mut c_void) };
    }
}

// Global storage of allocated buffer pointers.  A simple Vec is sufficient
// because we only ever store raw pointers and occasionally remove them.
static BUFFERS: OnceLock<Mutex<Vec<usize>>> = OnceLock::new();

// Counter similar to Vim's 'top_file_num', used by get_highest_fnum().
static TOP_FILE_NUM: AtomicI32 = AtomicI32::new(1);

fn allocate(size: usize) -> *mut FileBuffer {
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

#[deprecated(note = "use buf_alloc_owned instead")]
#[no_mangle]
pub extern "C" fn buf_alloc(size: usize) -> *mut FileBuffer {
    allocate(size)
}

#[no_mangle]
pub extern "C" fn buf_alloc_owned(size: usize) -> OwnedFileBuffer {
    OwnedFileBuffer(allocate(size))
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

#[no_mangle]
pub extern "C" fn calc_percentage(part: c_long, whole: c_long) -> c_int {
    if whole == 0 {
        return 0;
    }
    if part > 1_000_000 {
        (part / (whole / 100)) as c_int
    } else {
        ((part * 100) / whole) as c_int
    }
}

#[no_mangle]
pub extern "C" fn get_highest_fnum() -> c_int {
    TOP_FILE_NUM.load(Ordering::SeqCst) - 1
}

// Helper for tests to set the top file number.
#[cfg(test)]
fn set_top_file_num(num: i32) {
    TOP_FILE_NUM.store(num, Ordering::SeqCst);
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

    #[test]
    fn owned_alloc_and_drop() {
        let buf = buf_alloc_owned(16);
        let ptr = buf.as_ptr();
        assert!(!ptr.is_null());
        let list = BUFFERS.get().unwrap();
        assert!(list
            .lock()
            .unwrap()
            .contains(&(ptr as *mut c_void as usize)));
        drop(buf);
        assert!(!list
            .lock()
            .unwrap()
            .contains(&(ptr as *mut c_void as usize)));
    }

    #[test]
    fn percentage_calculation() {
        assert_eq!(calc_percentage(50, 200), 25);
        assert_eq!(calc_percentage(1_000_001, 2_000_000), 50);
    }

    #[test]
    fn highest_fnum() {
        set_top_file_num(10);
        assert_eq!(get_highest_fnum(), 9);
    }
}
