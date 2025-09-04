use rust_hashtab::*;
use std::ffi::CString;
use std::os::raw::c_void;

#[test]
fn hashtab_basic_operations() {
    unsafe {
        let tab = rust_hashtab_new();
        let key = CString::new("alpha").unwrap();
        let value = 0xdeadbeef as *mut c_void;
        assert_eq!(rust_hashtab_len(tab), 0);
        assert_eq!(rust_hashtab_get(tab, key.as_ptr()), std::ptr::null_mut());
        assert_eq!(rust_hashtab_set(tab, key.as_ptr(), value), 1);
        assert_eq!(rust_hashtab_len(tab), 1);
        assert_eq!(rust_hashtab_get(tab, key.as_ptr()), value);
        assert_eq!(rust_hashtab_remove(tab, key.as_ptr()), 1);
        assert_eq!(rust_hashtab_len(tab), 0);
        rust_hashtab_free(tab);
    }
}
