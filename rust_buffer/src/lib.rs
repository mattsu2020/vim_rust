use std::os::raw::{c_int, c_long, c_void};
use std::ptr::NonNull;
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

unsafe impl Send for FileBuffer {}
unsafe impl Sync for FileBuffer {}

// Global storage of allocated buffer pointers.  We wrap the Mutex in a newtype
// so that we can provide manual `Send` and `Sync` implementations even though
// `NonNull<T>` itself does not implement these traits.
struct BufferList(Mutex<Vec<NonNull<FileBuffer>>>);

unsafe impl Send for BufferList {}
unsafe impl Sync for BufferList {}

static BUFFERS: OnceLock<BufferList> = OnceLock::new();

// Counter similar to Vim's 'top_file_num', used by get_highest_fnum().
static TOP_FILE_NUM: AtomicI32 = AtomicI32::new(1);

// Safe wrappers around libc allocation routines using NonNull for safety.
fn calloc_file_buffer(size: usize) -> Option<NonNull<FileBuffer>> {
    let ptr = unsafe { libc::calloc(1, size) } as *mut FileBuffer;
    NonNull::new(ptr)
}

fn free_file_buffer(ptr: NonNull<FileBuffer>) {
    unsafe { libc::free(ptr.as_ptr() as *mut c_void) };
}

#[no_mangle]
pub extern "C" fn buf_alloc(size: usize) -> *mut FileBuffer {
    if let Some(ptr) = calloc_file_buffer(size) {
        BUFFERS
            .get_or_init(|| BufferList(Mutex::new(Vec::new())))
            .0
            .lock()
            .unwrap()
            .push(ptr);
        ptr.as_ptr()
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn buf_free(buf: *mut FileBuffer) {
    let Some(ptr) = NonNull::new(buf) else {
        return;
    };
    if let Some(m) = BUFFERS.get() {
        let mut buffers = m.0.lock().unwrap();
        buffers.retain(|&p| p != ptr);
    }
    free_file_buffer(ptr);
}

#[no_mangle]
pub extern "C" fn buf_freeall(_buf: *mut FileBuffer, _flags: c_int) {
    if let Some(m) = BUFFERS.get() {
        let mut buffers = m.0.lock().unwrap();
        for ptr in buffers.drain(..) {
            free_file_buffer(ptr);
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
    use std::ptr::NonNull;

    #[test]
    fn alloc_and_free() {
        let p = buf_alloc(16);
        assert!(!p.is_null());
        let list = &BUFFERS.get().unwrap().0;
        assert!(list.lock().unwrap().contains(&NonNull::new(p).unwrap()));
        buf_free(p);
        assert!(!list.lock().unwrap().contains(&NonNull::new(p).unwrap()));
    }

    #[test]
    fn multiple_allocations_and_free_all() {
        let p1 = buf_alloc(8);
        let p2 = buf_alloc(8);
        assert!(!p1.is_null() && !p2.is_null());
        let list = &BUFFERS.get().unwrap().0;
        {
            let guard = list.lock().unwrap();
            assert!(guard.contains(&NonNull::new(p1).unwrap()));
            assert!(guard.contains(&NonNull::new(p2).unwrap()));
        }
        buf_freeall(std::ptr::null_mut(), 0);
        assert!(list.lock().unwrap().is_empty());
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
