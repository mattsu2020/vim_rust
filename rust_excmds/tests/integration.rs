use rust_excmds::{
    ex_ascii,
    ex_mark_changed,
    rs_cmd_add,
    rs_cmd_execute,
};
use rust_change::Buffer;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

#[test]
fn ascii_matches_legacy() {
    let ptr = ex_ascii('A' as c_int);
    let cstr = unsafe { CStr::from_ptr(ptr) };
    assert_eq!(cstr.to_str().unwrap(), "<A>  65,  Hex 41,  Octal 101");
}

#[test]
fn mark_changed_sets_flag() {
    let mut b = Buffer::new(true);
    assert!(!b.changed);
    unsafe { ex_mark_changed(&mut b) };
    assert!(b.changed);
}

static mut CALLED: bool = false;

unsafe extern "C" fn dummy() {
    CALLED = true;
}

#[test]
fn command_table_runs_function() {
    unsafe { CALLED = false };
    rs_cmd_add(b"test\0".as_ptr() as *const c_char, dummy);
    let res = rs_cmd_execute(b"test\0".as_ptr() as *const c_char);
    assert_eq!(res, 1);
    assert!(unsafe { CALLED });
}

