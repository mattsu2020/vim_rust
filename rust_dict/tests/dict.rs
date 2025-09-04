use rust_dict::*;
use std::ffi::CString;
use std::os::raw::c_void;

#[test]
fn dict_basic_operations() {
    unsafe {
        let d = rust_dict_new();
        assert_eq!(rust_dict_len(d), 0);
        let key = CString::new("key").unwrap();
        let val = 0xdeadbeef as *mut c_void;
        assert_eq!(rust_dict_set(d, key.as_ptr(), val), 1);
        assert_eq!(rust_dict_len(d), 1);
        assert_eq!(rust_dict_get(d, key.as_ptr()), val);
        assert_eq!(rust_dict_remove(d, key.as_ptr()), 1);
        assert_eq!(rust_dict_len(d), 0);
        rust_dict_free(d);
    }
}
