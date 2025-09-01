use libc::{c_char, c_int, c_void};
use std::collections::HashMap;
use std::ffi::{CStr};
use std::sync::Mutex;

#[repr(C)]
pub struct typval_T { _private: [u8; 0] }

pub type CFunc = unsafe extern "C" fn(argc: c_int, argv: *mut typval_T, rettv: *mut typval_T, state: *mut c_void) -> c_int;
pub type CFuncFree = unsafe extern "C" fn(state: *mut c_void);

struct Entry {
    cb: CFunc,
    free: Option<CFuncFree>,
    state: *mut c_void,
}

unsafe impl Send for Entry {}
unsafe impl Sync for Entry {}

lazy_static::lazy_static! {
    static ref TABLE: Mutex<HashMap<String, Entry>> = Mutex::new(HashMap::new());
}

#[no_mangle]
pub extern "C" fn rust_userfunc_register(
    name: *const c_char,
    cb: CFunc,
    free_cb: Option<CFuncFree>,
    state: *mut c_void,
) -> c_int {
    if name.is_null() {
        return 0;
    }
    let cname = unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned();
    let entry = Entry { cb, free: free_cb, state };
    TABLE.lock().unwrap().insert(cname, entry);
    1
}

#[no_mangle]
pub extern "C" fn rust_userfunc_call(
    name: *const c_char,
    argc: c_int,
    argv: *mut typval_T,
    rettv: *mut typval_T,
) -> c_int {
    if name.is_null() {
        return 0;
    }
    let cname = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    let table = TABLE.lock().unwrap();
    if let Some(entry) = table.get(cname.as_ref()) {
        unsafe { (entry.cb)(argc, argv, rettv, entry.state) }
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn rust_userfunc_clear(name: *const c_char) {
    if name.is_null() {
        return;
    }
    let cname = unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned();
    if let Some(entry) = TABLE.lock().unwrap().remove(&cname) {
        if let Some(f) = entry.free {
            unsafe { f(entry.state) };
        }
    }
}
