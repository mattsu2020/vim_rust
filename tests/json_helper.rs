mod json {
    #![allow(dead_code)]
    include!("../src/json.rs");

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::ffi::CString;
        use std::os::raw::c_char;

        #[test]
        fn null_pointer_returns_none() {
            assert!(cstr_to_str(std::ptr::null()).is_none());
        }

        #[test]
        fn invalid_utf8_returns_none() {
            let bytes = [0xffu8, 0];
            let ptr = bytes.as_ptr() as *const c_char;
            assert!(cstr_to_str(ptr).is_none());
        }

        #[test]
        fn valid_utf8_returns_some() {
            let c = CString::new("hello").unwrap();
            let ptr = c.as_ptr();
            assert_eq!(cstr_to_str(ptr), Some("hello"));
        }
    }
}
