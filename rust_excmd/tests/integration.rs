use rust_excmd::{ex_ascii, ex_mark_changed};
use rust_change::Buffer;
use std::ffi::CStr;
use std::os::raw::c_int;

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
