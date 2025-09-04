use std::os::raw::{c_char, c_int, c_void};

#[no_mangle]
pub extern "C" fn LoadXpmImage(_filename: *const c_char, _h_image: *mut *mut c_void, _h_shape: *mut *mut c_void) -> c_int {
    -1
}
