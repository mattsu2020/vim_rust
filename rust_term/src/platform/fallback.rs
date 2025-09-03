use std::os::raw::c_int;

pub unsafe fn get_winsize(_width: *mut c_int, _height: *mut c_int) -> c_int {
    -1
}
