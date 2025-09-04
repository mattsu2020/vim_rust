use rust_cmdhist::{
    cmd_history_add, cmd_history_clear, cmd_history_get, cmd_history_init, cmd_history_len,
};
use std::ffi::CStr;
use std::os::raw::c_char;

#[test]
fn integration_add_get_and_len() {
    cmd_history_init(2);
    cmd_history_add(b"cmd1\0".as_ptr() as *const c_char);
    cmd_history_add(b"cmd2\0".as_ptr() as *const c_char);
    assert_eq!(cmd_history_len(), 2);
    cmd_history_add(b"cmd3\0".as_ptr() as *const c_char);
    assert_eq!(cmd_history_len(), 2);
    let first = unsafe { CStr::from_ptr(cmd_history_get(0)) };
    assert_eq!(first.to_str().unwrap(), "cmd2");
    cmd_history_clear();
    assert_eq!(cmd_history_len(), 0);
}
