use std::os::raw::{c_char, c_int};

#[no_mangle]
pub extern "C" fn rs_hardcopy_print(text: *const c_char) -> c_int {
    if text.is_null() {
        return -1;
    }
    unsafe {
        libc::puts(text);
    }
    0
}
