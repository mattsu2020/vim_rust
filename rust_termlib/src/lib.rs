use std::os::raw::c_int;

#[no_mangle]
pub extern "C" fn rs_termlib_clear() -> c_int {
    unsafe {
        libc::printf(b"\x1B[2J\x1B[H\0".as_ptr() as *const _);
    }
    0
}
