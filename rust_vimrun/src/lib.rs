use std::os::raw::{c_char, c_int};

#[no_mangle]
pub extern "C" fn rs_vimrun(cmd: *const c_char) -> c_int {
    if cmd.is_null() {
        return -1;
    }
    #[cfg(windows)]
    unsafe {
        winapi::um::shellapi::ShellExecuteA(
            std::ptr::null_mut(),
            b"open\0".as_ptr() as *const i8,
            cmd,
            std::ptr::null(),
            std::ptr::null(),
            winapi::um::winuser::SW_SHOWNORMAL,
        );
    }
    #[cfg(not(windows))]
    unsafe {
        libc::system(cmd);
    }
    0
}
