use rust_map::{rs_map_add, rs_map_clear, rs_map_lookup};
use std::ffi::CStr;
use std::os::raw::c_char;

#[test]
fn add_and_lookup_integration() {
    rs_map_clear();
    rs_map_add(
        b"jj\0".as_ptr() as *const c_char,
        b"<Esc>\0".as_ptr() as *const c_char,
    );
    let ptr = rs_map_lookup(b"jj\0".as_ptr() as *const c_char);
    let cstr = unsafe { CStr::from_ptr(ptr) };
    assert_eq!(cstr.to_str().unwrap(), "<Esc>");
}

#[test]
fn lookup_missing_returns_null() {
    rs_map_clear();
    let ptr = rs_map_lookup(b"zz\0".as_ptr() as *const c_char);
    assert!(ptr.is_null());
}
