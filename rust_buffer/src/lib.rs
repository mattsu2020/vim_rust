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

impl Drop for FileBuffer {
    fn drop(&mut self) {
        // `drop` is only called for pointers previously allocated by
        // `buf_alloc`, which uses `calloc_file_buffer` and registers the
        // pointer in `BUFFERS`.  At this point the pointer is still valid and
        // uniquely owned so calling `free_file_buffer` is safe.
        free_file_buffer(NonNull::from(self));
    }
}

// Information tracked for each allocated buffer.
#[derive(Clone, Copy)]
struct BufferRecord {
    ptr: NonNull<FileBuffer>,
    fnum: i32,
}

// Global storage of allocated buffers.  We wrap the Mutex in a newtype so that
// we can provide manual `Send` and `Sync` implementations even though
// `NonNull<T>` itself does not implement these traits.
struct BufferList(Mutex<Vec<BufferRecord>>);

unsafe impl Send for BufferList {}
unsafe impl Sync for BufferList {}

static BUFFERS: OnceLock<BufferList> = OnceLock::new();

// Counter similar to Vim's 'top_file_num', used by get_highest_fnum().
static TOP_FILE_NUM: AtomicI32 = AtomicI32::new(1);
static BUF_FREE_COUNT: AtomicI32 = AtomicI32::new(0);

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
        let rec = BufferRecord {
            ptr,
            fnum: next_top_file_num(),
        };
        BUFFERS
            .get_or_init(|| BufferList(Mutex::new(Vec::new())))
            .0
            .lock()
            .unwrap()
            .push(rec);
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
        buffers.retain(|rec| rec.ptr != ptr);
    }
    inc_buf_free_count();
    // SAFETY: `ptr` was allocated by `buf_alloc`.  `drop_in_place` will invoke
    // `FileBuffer`'s `Drop` implementation which releases the allocation.
    unsafe {
        std::ptr::drop_in_place(ptr.as_ptr());
    }
}

#[no_mangle]
pub extern "C" fn buf_freeall(_buf: *mut FileBuffer, _flags: c_int) {
    if let Some(m) = BUFFERS.get() {
        let mut buffers = m.0.lock().unwrap();
        for rec in buffers.drain(..) {
            inc_buf_free_count();
            // SAFETY: the pointers in `buffers` originate from `buf_alloc` and
            // are unique.  Dropping them releases the memory.
            unsafe {
                std::ptr::drop_in_place(rec.ptr.as_ptr());
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn calc_percentage(part: c_long, whole: c_long) -> c_int {
    if whole <= 0 {
        return 0;
    }
    if part > 1_000_000 && whole >= 100 {
        (part / (whole / 100)) as c_int
    } else {
        ((part * 100) / whole) as c_int
    }
}

#[no_mangle]
pub extern "C" fn get_highest_fnum() -> c_int {
    TOP_FILE_NUM.load(Ordering::SeqCst) - 1
}

#[no_mangle]
pub extern "C" fn get_top_file_num() -> c_int {
    TOP_FILE_NUM.load(Ordering::SeqCst)
}

#[no_mangle]
pub extern "C" fn set_top_file_num(num: c_int) {
    TOP_FILE_NUM.store(num, Ordering::SeqCst);
}

#[no_mangle]
pub extern "C" fn next_top_file_num() -> c_int {
    TOP_FILE_NUM.fetch_add(1, Ordering::SeqCst)
}

#[no_mangle]
pub extern "C" fn dec_top_file_num() {
    TOP_FILE_NUM.fetch_sub(1, Ordering::SeqCst);
}

#[no_mangle]
pub extern "C" fn get_buf_free_count() -> c_int {
    BUF_FREE_COUNT.load(Ordering::SeqCst)
}

#[no_mangle]
pub extern "C" fn inc_buf_free_count() {
    BUF_FREE_COUNT.fetch_add(1, Ordering::SeqCst);
}

// Helper: look up a buffer record for a raw pointer.
fn find_buffer(ptr: NonNull<FileBuffer>) -> Option<BufferRecord> {
    BUFFERS
        .get()
        .and_then(|m| m.0.lock().ok()?.iter().find(|rec| rec.ptr == ptr).cloned())
}

#[repr(C)]
pub struct BufRef {
    pub br_buf: *mut FileBuffer,
    pub br_fnum: c_int,
    pub br_buf_free_count: c_int,
}

#[no_mangle]
pub extern "C" fn set_bufref(bufref: *mut BufRef, buf: *mut FileBuffer) {
    if bufref.is_null() {
        return;
    }
    let mut br = unsafe { &mut *bufref };
    br.br_buf = buf;
    if let Some(rec) = NonNull::new(buf).and_then(find_buffer) {
        br.br_fnum = rec.fnum;
    } else {
        br.br_fnum = 0;
    }
    br.br_buf_free_count = get_buf_free_count();
}

#[no_mangle]
pub extern "C" fn buf_valid(buf: *mut FileBuffer) -> c_int {
    let Some(ptr) = NonNull::new(buf) else {
        return 0;
    };
    if find_buffer(ptr).is_some() { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn bufref_valid(bufref: *const BufRef) -> c_int {
    if bufref.is_null() {
        return 0;
    }
    let br = unsafe { &*bufref };
    if br.br_buf_free_count == get_buf_free_count() {
        return 1;
    }
    let Some(ptr) = NonNull::new(br.br_buf) else {
        return 0;
    };
    match find_buffer(ptr) {
        Some(rec) if rec.fnum == br.br_fnum => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{ptr::NonNull, sync::atomic::Ordering};

    #[test]
    fn alloc_and_free() {
        let p = buf_alloc(16);
        assert!(!p.is_null());
        let list = &BUFFERS.get().unwrap().0;
        assert!(list
            .lock()
            .unwrap()
            .iter()
            .any(|rec| rec.ptr == NonNull::new(p).unwrap()));
        buf_free(p);
        assert!(!list
            .lock()
            .unwrap()
            .iter()
            .any(|rec| rec.ptr == NonNull::new(p).unwrap()));
    }

    #[test]
    fn multiple_allocations_and_free_all() {
        let p1 = buf_alloc(8);
        let p2 = buf_alloc(8);
        assert!(!p1.is_null() && !p2.is_null());
        let list = &BUFFERS.get().unwrap().0;
        {
            let guard = list.lock().unwrap();
            assert!(guard.iter().any(|rec| rec.ptr == NonNull::new(p1).unwrap()));
            assert!(guard.iter().any(|rec| rec.ptr == NonNull::new(p2).unwrap()));
        }
        buf_freeall(std::ptr::null_mut(), 0);
        assert!(list.lock().unwrap().is_empty());
    }

    #[test]
    fn bufref_tracking() {
        let p = buf_alloc(4);
        assert!(buf_valid(p) != 0);
        let mut br = BufRef { br_buf: std::ptr::null_mut(), br_fnum: 0, br_buf_free_count: 0 };
        set_bufref(&mut br, p);
        assert!(bufref_valid(&br) != 0);
        buf_free(p);
        assert!(buf_valid(p) == 0);
        assert!(bufref_valid(&br) == 0);
    }

    #[test]
    fn percentage_calculation() {
        assert_eq!(calc_percentage(50, 200), 25);
        assert_eq!(calc_percentage(1_000_001, 2_000_000), 50);
        assert_eq!(calc_percentage(1_000_001, 50), 2_000_002);
    }

    #[test]
    fn highest_fnum() {
        set_top_file_num(10);
        assert_eq!(get_highest_fnum(), 9);
    }

    #[test]
    fn buf_free_count_tracking() {
        super::BUF_FREE_COUNT.store(0, Ordering::SeqCst);
        assert_eq!(get_buf_free_count(), 0);
        inc_buf_free_count();
        assert_eq!(get_buf_free_count(), 1);
    }
}
