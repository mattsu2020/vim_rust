use libc::{c_char, c_int, c_uchar};
use rust_strings::*;
use std::ffi::{CStr, CString};

#[test]
fn test_skip_and_copy_option() {
    unsafe {
        let s = CString::new(",  test").unwrap();
        let p = skip_to_option_part(s.as_ptr() as *mut c_uchar);
        let res = CStr::from_ptr(p as *const c_char).to_str().unwrap();
        assert_eq!(res, "test");

        let opt = CString::new("part1, part2").unwrap();
        let p = opt.as_ptr() as *mut c_uchar;
        let mut buf = [0u8; 20];
        let mut option_ptr = p;
        let len = copy_option_part(
            &mut option_ptr,
            buf.as_mut_ptr(),
            buf.len() as c_int,
            CString::new(",").unwrap().as_ptr(),
        );
        assert_eq!(len, 5);
        let part = CStr::from_ptr(buf.as_ptr() as *const c_char)
            .to_str()
            .unwrap();
        assert_eq!(part, "part1");
    }
}

#[test]
fn test_vim_isspace() {
    assert_eq!(vim_isspace(b' ' as c_int), 1);
    assert_eq!(vim_isspace(9), 1);
    assert_eq!(vim_isspace(b'a' as c_int), 0);
}

#[test]
fn test_vim_strsave_and_strnsave() {
    unsafe {
        let s = CString::new("hello").unwrap();
        let p = vim_strsave(s.as_ptr() as *const c_uchar);
        let res = CStr::from_ptr(p as *const c_char).to_str().unwrap();
        assert_eq!(res, "hello");
        libc::free(p as *mut libc::c_void);

        let p2 = vim_strnsave(s.as_ptr() as *const c_uchar, 3);
        let res2 = CStr::from_ptr(p2 as *const c_char).to_str().unwrap();
        assert_eq!(res2, "hel");
        libc::free(p2 as *mut libc::c_void);
    }
}
