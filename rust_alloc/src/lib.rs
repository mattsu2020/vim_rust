use std::os::raw::{c_int, c_void};
use std::mem;

const SIZE_HEADER: usize = mem::size_of::<usize>();

unsafe fn alloc_block(size: usize, zero: bool) -> *mut c_void {
    if size == 0 {
        return std::ptr::null_mut();
    }
    let total = size + SIZE_HEADER;
    let mut vec = if zero {
        vec![0u8; total]
    } else {
        let mut v = Vec::<u8>::with_capacity(total);
        v.set_len(total);
        v
    };
    let ptr = vec.as_mut_ptr();
    *(ptr as *mut usize) = size;
    let data_ptr = ptr.add(SIZE_HEADER);
    mem::forget(vec);
    data_ptr as *mut c_void
}

#[no_mangle]
pub extern "C" fn lalloc(size: usize, _message: c_int) -> *mut c_void {
    unsafe { alloc_block(size, false) }
}

#[no_mangle]
pub extern "C" fn lalloc_clear(size: usize, _message: c_int) -> *mut c_void {
    unsafe { alloc_block(size, true) }
}

#[no_mangle]
pub extern "C" fn lalloc_id(size: usize, message: c_int, _id: usize) -> *mut c_void {
    lalloc(size, message)
}

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    lalloc(size, 1)
}

#[no_mangle]
pub extern "C" fn alloc_id(size: usize, _id: usize) -> *mut c_void {
    alloc(size)
}

#[no_mangle]
pub extern "C" fn alloc_clear(size: usize) -> *mut c_void {
    lalloc_clear(size, 1)
}

#[no_mangle]
pub extern "C" fn alloc_clear_id(size: usize, _id: usize) -> *mut c_void {
    alloc_clear(size)
}

#[no_mangle]
pub extern "C" fn vim_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let size_ptr = (ptr as *mut u8).offset(-(SIZE_HEADER as isize));
        let total = *(size_ptr as *mut usize) + SIZE_HEADER;
        let slice = std::slice::from_raw_parts_mut(size_ptr, total);
        drop(Box::from_raw(slice));
    }
}
