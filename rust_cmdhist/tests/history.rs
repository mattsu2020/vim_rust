use rust_cmdhist::{
    rs_cmd_history_add, rs_cmd_history_clear, rs_cmd_history_get, rs_cmd_history_init,
    rs_cmd_history_len,
};
use std::ffi::CStr;
use std::os::raw::c_char;

#[test]
fn integration_add_get_and_len() {
    rs_cmd_history_init(2);
    rs_cmd_history_add(b"cmd1\0".as_ptr() as *const c_char);
    rs_cmd_history_add(b"cmd2\0".as_ptr() as *const c_char);
    assert_eq!(rs_cmd_history_len(), 2);
    rs_cmd_history_add(b"cmd3\0".as_ptr() as *const c_char);
    assert_eq!(rs_cmd_history_len(), 2);
    let first = unsafe { CStr::from_ptr(rs_cmd_history_get(0)) };
    assert_eq!(first.to_str().unwrap(), "cmd2");
    rs_cmd_history_clear();
    assert_eq!(rs_cmd_history_len(), 0);
}
