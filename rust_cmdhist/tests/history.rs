use rust_cmdhist::{rs_cmd_history_add, rs_cmd_history_clear, rs_cmd_history_get};
use std::ffi::CStr;
use std::os::raw::c_char;

#[test]
fn integration_add_get() {
    rs_cmd_history_clear();
    rs_cmd_history_add(b"cmd1\0".as_ptr() as *const c_char);
    let ptr = rs_cmd_history_get(0);
    let cstr = unsafe { CStr::from_ptr(ptr) };
    assert_eq!(cstr.to_str().unwrap(), "cmd1");
}
