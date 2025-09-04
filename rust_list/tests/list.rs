use rust_list::*;
use std::os::raw::c_void;

#[test]
fn list_basic_operations() {
    unsafe {
        let list = rust_list_new();
        assert_eq!(rust_list_len(list), 0);
        rust_list_append(list, 1 as *mut c_void);
        rust_list_append(list, 2 as *mut c_void);
        assert_eq!(rust_list_len(list), 2);
        assert_eq!(rust_list_get(list, 0), 1 as *mut c_void);
        assert_eq!(rust_list_remove(list, 0), 1 as *mut c_void);
        assert_eq!(rust_list_len(list), 1);
        rust_list_free(list);
    }
}
