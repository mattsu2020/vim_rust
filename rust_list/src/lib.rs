#![allow(clippy::missing_safety_doc)]
use std::os::raw::c_void;

#[repr(C)]
pub struct VimList {
    items: Vec<*mut c_void>,
}

#[no_mangle]
pub extern "C" fn rust_list_new() -> *mut VimList {
    Box::into_raw(Box::new(VimList { items: Vec::new() }))
}

#[no_mangle]
pub unsafe extern "C" fn rust_list_free(l: *mut VimList) {
    if !l.is_null() {
        drop(Box::from_raw(l));
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_list_len(l: *const VimList) -> usize {
    l.as_ref().map(|list| list.items.len()).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn rust_list_append(l: *mut VimList, item: *mut c_void) {
    if let Some(list) = l.as_mut() {
        list.items.push(item);
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_list_get(l: *const VimList, index: usize) -> *mut c_void {
    if let Some(list) = l.as_ref() {
        list.items
            .get(index)
            .copied()
            .unwrap_or(std::ptr::null_mut())
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_list_remove(l: *mut VimList, index: usize) -> *mut c_void {
    if let Some(list) = l.as_mut() {
        if index < list.items.len() {
            return list.items.remove(index);
        }
    }
    std::ptr::null_mut()
}
