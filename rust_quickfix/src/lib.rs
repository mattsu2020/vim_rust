use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_long, c_void};
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[repr(C)]
pub struct typval_T { _private: [u8; 0] }

#[derive(Default)]
struct QfEntry {
    text: String,
    lnum: i64,
    col: i32,
}

static LIST: Lazy<Mutex<Vec<QfEntry>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Clear the current quickfix list, creating a new empty list.
#[no_mangle]
pub extern "C" fn rs_qf_new_list() {
    let mut list = LIST.lock().unwrap();
    list.clear();
}

#[no_mangle]
pub extern "C" fn rs_qf_add_entry(
    _qfl: *mut c_void,
    _dir: *const c_char,
    _fname: *const c_char,
    _module: *const c_char,
    _bufnum: c_int,
    mesg: *const c_char,
    lnum: c_long,
    _end_lnum: c_long,
    col: c_int,
    _end_col: c_int,
    _vis_col: c_int,
    _pattern: *const c_char,
    _nr: c_int,
    _typ: c_int,
    _user_data: *mut typval_T,
    _valid: c_int,
) -> c_int {
    if mesg.is_null() {
        return 0; // QF_FAIL
    }
    let c_str = unsafe { CStr::from_ptr(mesg) };
    let text = match c_str.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return 0,
    };
    let mut list = LIST.lock().unwrap();
    list.push(QfEntry { text, lnum: lnum as i64, col });
    1 // QF_OK
}

#[no_mangle]
pub extern "C" fn rs_qf_list(_eap: *mut c_void) {
    let list = LIST.lock().unwrap();
    for (i, item) in list.iter().enumerate() {
        println!("{}: {}:{} {}", i + 1, item.lnum, item.col, item.text);
    }
}

/// Return the number of entries in the quickfix list.
#[no_mangle]
pub extern "C" fn rs_qf_len() -> c_int {
    let list = LIST.lock().unwrap();
    list.len() as c_int
}
#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::ffi::CString;
    use std::sync::Mutex;

    static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn add_and_list() {
        let _g = TEST_MUTEX.lock().unwrap();
        rs_qf_new_list();
        let msg = CString::new("test").unwrap();
        assert_eq!(
            rs_qf_add_entry(
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                msg.as_ptr(),
                1,
                0,
                0,
                0,
                0,
                std::ptr::null(),
                0,
                0,
                std::ptr::null_mut(),
                1,
            ),
            1
        );
        rs_qf_list(std::ptr::null_mut());
    }

    #[test]
    fn new_list_clears_entries() {
        let _g = TEST_MUTEX.lock().unwrap();
        rs_qf_new_list();
        let msg = CString::new("entry").unwrap();
        rs_qf_add_entry(
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            msg.as_ptr(),
            1,
            0,
            0,
            0,
            0,
            std::ptr::null(),
            0,
            0,
            std::ptr::null_mut(),
            1,
        );
        assert_eq!(rs_qf_len(), 1);
        rs_qf_new_list();
        assert_eq!(rs_qf_len(), 0);
    }
}
