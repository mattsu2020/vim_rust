use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CStr;
use std::fs;
use std::os::raw::c_char;
use std::sync::Mutex;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum Vartype {
    VAR_UNKNOWN = 0,
    VAR_NUMBER,
    VAR_STRING,
}

#[repr(C)]
pub union ValUnion {
    pub v_number: i64,
    pub v_string: *mut c_char,
}

#[repr(C)]
pub struct typval_T {
    pub v_type: Vartype,
    pub v_lock: c_char,
    pub vval: ValUnion,
}

#[derive(Clone)]
struct Buffer {
    lines: Vec<String>,
}

struct BufferManager {
    bufs: HashMap<String, Buffer>,
}

#[repr(C)]
pub struct buf_T {
    _private: [u8; 0],
}

static BUFFER_MANAGER: Lazy<Mutex<BufferManager>> = Lazy::new(|| {
    Mutex::new(BufferManager { bufs: HashMap::new() })
});

static DUMMY_BUF: buf_T = buf_T { _private: [] };

#[no_mangle]
pub extern "C" fn buflist_find_by_name_rs(name: *const c_char, _curtab_only: bool) -> *mut buf_T {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(name) };
    let name_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let mut manager = BUFFER_MANAGER.lock().unwrap();
    if !manager.bufs.contains_key(name_str) {
        let content = match fs::read_to_string(name_str) {
            Ok(c) => c,
            Err(_) => return std::ptr::null_mut(),
        };
        let buf = Buffer {
            lines: content.lines().map(|l| l.to_string()).collect(),
        };
        manager.bufs.insert(name_str.to_string(), buf);
    }
    &DUMMY_BUF as *const buf_T as *mut buf_T
}

