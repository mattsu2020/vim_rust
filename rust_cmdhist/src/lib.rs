use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::Mutex;

use once_cell::sync::Lazy;

pub use rust_usercmd::{rs_user_command_delete, rs_user_command_register};

static HISTORY: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[no_mangle]
pub extern "C" fn rs_cmd_history_add(cmd: *const c_char) {
    let cmd = unsafe { CStr::from_ptr(cmd) }.to_string_lossy().into_owned();
    HISTORY.lock().unwrap().push(cmd);
}

#[no_mangle]
pub extern "C" fn rs_cmd_history_get(idx: c_int) -> *const c_char {
    let hist = HISTORY.lock().unwrap();
    if let Some(cmd) = hist.get(idx as usize) {
        CString::new(cmd.clone()).unwrap().into_raw()
    } else {
        std::ptr::null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn add_and_get() {
        rs_cmd_history_add(b"cmd1\0".as_ptr() as *const c_char);
        let ptr = rs_cmd_history_get(0);
        let cstr = unsafe { CStr::from_ptr(ptr) };
        assert_eq!(cstr.to_str().unwrap(), "cmd1");
    }

    #[test]
    fn register_and_delete_command() {
        let name = CString::new("MyCmd").unwrap();
        let rep = CString::new("echo").unwrap();
        let res = rs_user_command_register(
            name.as_ptr(),
            rep.as_ptr(),
            0,
            0,
            0,
            0,
            std::ptr::null(),
            0,
            0,
        );
        assert_eq!(res, 0);
        assert_eq!(rs_user_command_delete(name.as_ptr()), 0);
    }
}
