#![allow(unsafe_op_in_unsafe_fn)]

use std::collections::HashMap;
use std::os::raw::{c_char, c_int, c_uchar, c_void};
use std::sync::{Mutex, OnceLock};

use libc::{memmove, memset, strlen, strcpy};

// Constants used throughout Vim's C code
const OK: c_int = 1;
const FAIL: c_int = 0;

// Equivalent of the "alloc_id_T" enumeration.  The actual values are
// irrelevant on the Rust side, so use a plain integer type.
pub type alloc_id_T = c_int;

// Structure representing a growing array, copied from structs.h
#[repr(C)]
pub struct garray_T {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

// Lazily initialized global map of allocations so that memory obtained in
// Rust can later be released from C code.
static ALLOCATIONS: OnceLock<Mutex<HashMap<usize, Vec<u8>>>> = OnceLock::new();

fn allocations() -> &'static Mutex<HashMap<usize, Vec<u8>>> {
    ALLOCATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn alloc_impl(size: usize) -> *mut c_void {
    let mut buf: Vec<u8> = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    // Safety: reserve `size` bytes and assume caller initialises them.
    unsafe { buf.set_len(size); }
    allocations().lock().unwrap().insert(ptr as usize, buf);
    ptr as *mut c_void
}

fn alloc_clear_impl(size: usize) -> *mut c_void {
    let mut buf = vec![0u8; size];
    let ptr = buf.as_mut_ptr();
    allocations().lock().unwrap().insert(ptr as usize, buf);
    ptr as *mut c_void
}

fn free_impl(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    allocations().lock().unwrap().remove(&(ptr as usize));
}

fn mem_realloc_impl(ptr: *mut c_void, size: usize) -> *mut c_void {
    if ptr.is_null() {
        return alloc_impl(size);
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
        // Pointer not tracked (likely not from alloc_impl). Allocate fresh.
        alloc_impl(size)
    }
}

unsafe fn vim_strsave(p: *const c_uchar) -> *mut c_uchar {
    let len = strlen(p as *const c_char) as usize;
    let s = alloc_impl(len + 1) as *mut c_uchar;
    if !s.is_null() {
        memmove(s as *mut c_void, p as *const c_void, len + 1);
    }
    s
}

// -- exported functions ----------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn vim_mem_profile_dump() {}

#[no_mangle]
pub unsafe extern "C" fn alloc_does_fail(_size: usize) -> c_int { FAIL }

#[no_mangle]
pub unsafe extern "C" fn alloc(size: usize) -> *mut c_void {
    alloc_impl(size)
}

#[no_mangle]
pub unsafe extern "C" fn alloc_id(size: usize, _id: alloc_id_T) -> *mut c_void {
    alloc_impl(size)
}

#[no_mangle]
pub unsafe extern "C" fn alloc_clear(size: usize) -> *mut c_void {
    alloc_clear_impl(size)
}

#[no_mangle]
pub unsafe extern "C" fn alloc_clear_id(size: usize, _id: alloc_id_T) -> *mut c_void {
    alloc_clear_impl(size)
}

#[no_mangle]
pub unsafe extern "C" fn lalloc_clear(size: usize, _message: c_int) -> *mut c_void {
    alloc_clear_impl(size)
}

#[no_mangle]
pub unsafe extern "C" fn lalloc(size: usize, _message: c_int) -> *mut c_void {
    alloc_impl(size)
}

#[no_mangle]
pub unsafe extern "C" fn lalloc_id(size: usize, message: c_int, _id: alloc_id_T) -> *mut c_void {
    lalloc(size, message)
}

#[no_mangle]
pub unsafe extern "C" fn mem_realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    mem_realloc_impl(ptr, size)
}

#[no_mangle]
pub unsafe extern "C" fn do_outofmem_msg(_size: usize) {}

#[no_mangle]
pub unsafe extern "C" fn free_all_mem() {}

#[no_mangle]
pub unsafe extern "C" fn vim_memsave(p: *const c_uchar, len: usize) -> *mut c_uchar {
    let ret = alloc_impl(len) as *mut c_uchar;
    if !ret.is_null() {
        memmove(ret as *mut c_void, p as *const c_void, len);
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn vim_free(x: *mut c_void) {
    free_impl(x);
}

#[no_mangle]
pub unsafe extern "C" fn ga_clear(gap: *mut garray_T) {
    vim_free((*gap).ga_data);
    ga_init(gap);
}

#[no_mangle]
pub unsafe extern "C" fn ga_clear_strings(gap: *mut garray_T) {
    if !(*gap).ga_data.is_null() {
        let data = (*gap).ga_data as *mut *mut c_uchar;
        for i in 0..(*gap).ga_len {
            let ptr = *data.add(i as usize);
            vim_free(ptr as *mut c_void);
        }
    }
    ga_clear(gap);
}

#[no_mangle]
pub unsafe extern "C" fn ga_copy_strings(from: *mut garray_T, to: *mut garray_T) -> c_int {
    ga_init2(to, std::mem::size_of::<*mut c_uchar>(), 1);
    if ga_grow(to, (*from).ga_len) == FAIL {
        return FAIL;
    }
    let from_data = (*from).ga_data as *mut *mut c_uchar;
    let to_data = (*to).ga_data as *mut *mut c_uchar;
    for i in 0..(*from).ga_len {
        let orig = *from_data.add(i as usize);
        let copy = if orig.is_null() {
            std::ptr::null_mut()
        } else {
            let len = strlen(orig as *const c_char) as usize;
            let dest = alloc_impl(len + 1) as *mut c_uchar;
            if dest.is_null() {
                (*to).ga_len = i;
                ga_clear_strings(to);
                return FAIL;
            }
            memmove(dest as *mut c_void, orig as *const c_void, len + 1);
            dest
        };
        *to_data.add(i as usize) = copy;
    }
    (*to).ga_len = (*from).ga_len;
    OK
}

#[no_mangle]
pub unsafe extern "C" fn ga_init(gap: *mut garray_T) {
    (*gap).ga_data = std::ptr::null_mut();
    (*gap).ga_maxlen = 0;
    (*gap).ga_len = 0;
}

#[no_mangle]
pub unsafe extern "C" fn ga_init2(gap: *mut garray_T, itemsize: usize, growsize: c_int) {
    ga_init(gap);
    (*gap).ga_itemsize = itemsize as c_int;
    (*gap).ga_growsize = growsize;
}

#[no_mangle]
pub unsafe extern "C" fn ga_grow(gap: *mut garray_T, n: c_int) -> c_int {
    if (*gap).ga_maxlen - (*gap).ga_len < n {
        return ga_grow_inner(gap, n);
    }
    OK
}

#[no_mangle]
pub unsafe extern "C" fn ga_grow_id(gap: *mut garray_T, n: c_int, _id: alloc_id_T) -> c_int {
    ga_grow_inner(gap, n)
}

#[no_mangle]
pub unsafe extern "C" fn ga_grow_inner(gap: *mut garray_T, mut n: c_int) -> c_int {
    if n < (*gap).ga_growsize {
        n = (*gap).ga_growsize;
    }

    if (*gap).ga_maxlen - (*gap).ga_len < n {
        if n < (*gap).ga_len / 2 {
            n = (*gap).ga_len / 2;
        }
        let new_len = ((*gap).ga_itemsize as usize) * ((*gap).ga_len + n) as usize;
        let pp = mem_realloc_impl((*gap).ga_data, new_len) as *mut c_uchar;
        if pp.is_null() {
            return FAIL;
        }
        let old_len = ((*gap).ga_itemsize as usize) * ((*gap).ga_maxlen as usize);
        memset(pp.add(old_len) as *mut c_void, 0, new_len - old_len);
        (*gap).ga_maxlen = (*gap).ga_len + n;
        (*gap).ga_data = pp as *mut c_void;
    }
    OK
}

#[no_mangle]
pub unsafe extern "C" fn ga_concat_strings(gap: *mut garray_T, sep: *const c_char) -> *mut c_uchar {
    let sep_len = strlen(sep) as usize;
    let data = (*gap).ga_data as *mut *mut c_uchar;
    let mut len: usize = 0;
    for i in 0..(*gap).ga_len {
        let s = *data.add(i as usize);
        len += strlen(s as *const c_char) as usize;
    }
    if (*gap).ga_len > 1 {
        len += ((*gap).ga_len - 1) as usize * sep_len;
    }
    let s = alloc_impl(len + 1) as *mut c_uchar;
    if s.is_null() {
        return std::ptr::null_mut();
    }
    *s = 0;
    let mut p = s;
    for i in 0..(*gap).ga_len {
        let item = *data.add(i as usize);
        if p != s {
            strcpy(p as *mut c_char, sep);
            p = p.add(sep_len);
        }
        strcpy(p as *mut c_char, item as *const c_char);
        p = p.add(strlen(p as *const c_char) as usize);
    }
    s
}

#[no_mangle]
pub unsafe extern "C" fn ga_copy_string(gap: *mut garray_T, p: *const c_uchar) -> c_int {
    let cp = vim_strsave(p);
    if cp.is_null() {
        return FAIL;
    }
    if ga_grow(gap, 1) == FAIL {
        vim_free(cp as *mut c_void);
        return FAIL;
    }
    let data = (*gap).ga_data as *mut *mut c_uchar;
    *data.add((*gap).ga_len as usize) = cp;
    (*gap).ga_len += 1;
    OK
}

#[no_mangle]
pub unsafe extern "C" fn ga_add_string(gap: *mut garray_T, p: *mut c_uchar) -> c_int {
    if ga_grow(gap, 1) == FAIL {
        return FAIL;
    }
    let data = (*gap).ga_data as *mut *mut c_uchar;
    *data.add((*gap).ga_len as usize) = p;
    (*gap).ga_len += 1;
    OK
}

#[no_mangle]
pub unsafe extern "C" fn ga_concat(gap: *mut garray_T, s: *const c_uchar) {
    if s.is_null() || *s == 0 {
        return;
    }
    let len = strlen(s as *const c_char) as usize;
    if ga_grow(gap, len as c_int) == OK {
        let dest = ((*gap).ga_data as *mut c_uchar).add((*gap).ga_len as usize);
        memmove(dest as *mut c_void, s as *const c_void, len);
        (*gap).ga_len += len as c_int;
    }
}

#[no_mangle]
pub unsafe extern "C" fn ga_concat_len(gap: *mut garray_T, s: *const c_uchar, len: usize) {
    if s.is_null() || *s == 0 || len == 0 {
        return;
    }
    if ga_grow(gap, len as c_int) == OK {
        let dest = ((*gap).ga_data as *mut c_uchar).add((*gap).ga_len as usize);
        memmove(dest as *mut c_void, s as *const c_void, len);
        (*gap).ga_len += len as c_int;
    }
}

#[no_mangle]
pub unsafe extern "C" fn ga_append(gap: *mut garray_T, c: c_int) -> c_int {
    if ga_grow(gap, 1) == FAIL {
        return FAIL;
    }
    let dest = ((*gap).ga_data as *mut c_uchar).add((*gap).ga_len as usize);
    *dest = c as c_uchar;
    (*gap).ga_len += 1;
    OK
}

#[no_mangle]
pub unsafe extern "C" fn append_ga_line(gap: *mut garray_T) {
    if ga_grow(gap, (*gap).ga_len + 1) == OK {
        let dest = ((*gap).ga_data as *mut c_uchar).add((*gap).ga_len as usize);
        *dest = 0;
        (*gap).ga_len += 1;
    }
}

// -- tests -----------------------------------------------------------------

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
