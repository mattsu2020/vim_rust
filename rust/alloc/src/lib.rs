use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "mem_profile")]
static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "mem_profile")]
static FREED: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
pub extern "C" fn rs_alloc(size: usize) -> *mut c_void {
    alloc_impl(size, false)
}

#[no_mangle]
pub extern "C" fn rs_alloc_zeroed(size: usize) -> *mut c_void {
    alloc_impl(size, true)
}

fn alloc_impl(size: usize, zeroed: bool) -> *mut c_void {
    let total = size + std::mem::size_of::<usize>();
    let mut v = if zeroed {
        vec![0u8; total]
    } else {
        let mut vec = Vec::with_capacity(total);
        unsafe { vec.set_len(total); }
        vec
    };

    let ptr = v.as_mut_ptr();
    unsafe {
        *(ptr as *mut usize) = size;
        #[cfg(feature = "mem_profile")]
        {
            ALLOCATED.fetch_add(size, Ordering::Relaxed);
        }
        let user = ptr.add(std::mem::size_of::<usize>());
        std::mem::forget(v);
        user as *mut c_void
    }
}

#[no_mangle]
pub unsafe extern "C" fn rs_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let base = (ptr as *mut u8).sub(std::mem::size_of::<usize>());
    let size = *(base as *const usize);
    let capacity = size + std::mem::size_of::<usize>();
    #[cfg(feature = "mem_profile")]
    {
        FREED.fetch_add(size, Ordering::Relaxed);
    }
    let _ = Vec::from_raw_parts(base, 0, capacity);
}

#[no_mangle]
pub unsafe extern "C" fn rs_realloc(ptr: *mut c_void, new_size: usize) -> *mut c_void {
    if ptr.is_null() {
        return rs_alloc(new_size);
    }
    let base = (ptr as *mut u8).sub(std::mem::size_of::<usize>());
    let old_size = *(base as *const usize);
    let old_total = old_size + std::mem::size_of::<usize>();
    let mut vec = Vec::from_raw_parts(base, old_total, old_total);
    vec.resize(new_size + std::mem::size_of::<usize>(), 0);

    let ptr = vec.as_mut_ptr();
    *(ptr as *mut usize) = new_size;
    #[cfg(feature = "mem_profile")]
    {
        if new_size > old_size {
            ALLOCATED.fetch_add(new_size - old_size, Ordering::Relaxed);
        } else {
            FREED.fetch_add(old_size - new_size, Ordering::Relaxed);
        }
    }
    let user = ptr.add(std::mem::size_of::<usize>());
    std::mem::forget(vec);
    user as *mut c_void
}

#[cfg(feature = "mem_profile")]
#[no_mangle]
pub extern "C" fn rs_mem_stats(allocated: *mut usize, freed: *mut usize) {
    unsafe {
        if !allocated.is_null() {
            *allocated = ALLOCATED.load(Ordering::Relaxed);
        }
        if !freed.is_null() {
            *freed = FREED.load(Ordering::Relaxed);
        }
    }
}
