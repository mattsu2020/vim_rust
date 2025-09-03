use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

#[no_mangle]
pub extern "C" fn rs_textformat_example(input: *const c_char) -> c_int {
    if input.is_null() {
        return -1;
    }
    let s = unsafe { CStr::from_ptr(input) };
    s.to_bytes().len() as c_int
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn example_works() {
        let input = CString::new("abc").unwrap();
        let len = rs_textformat_example(input.as_ptr());
        assert_eq!(len, 3);
    }
}
