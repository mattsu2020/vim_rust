include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Print a line of text via the underlying C helper.
pub fn print_line(s: &str) {
    let c_string = CString::new(s).expect("CString::new failed");
    unsafe {
        c_print_line(c_string.as_ptr());
    }
}

/// C ABI wrapper for printing a line.
#[no_mangle]
pub extern "C" fn rs_hardcopy_print_line(ptr: *const c_char) {
    if ptr.is_null() {
        return;
    }
    let c_str = unsafe { CStr::from_ptr(ptr) };
    if let Ok(text) = c_str.to_str() {
        print_line(text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        print_line("test");
    }
}
