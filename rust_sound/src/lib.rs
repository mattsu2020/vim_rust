use std::os::raw::c_int;

#[no_mangle]
pub extern "C" fn rs_sound_beep() -> c_int {
    #[cfg(windows)]
    unsafe {
        winapi::um::winuser::MessageBeep(winapi::um::winuser::MB_OK);
    }
    #[cfg(not(windows))]
    unsafe {
        libc::putchar(0x07 as c_int);
        libc::fflush(std::ptr::null_mut());
    }
    0
}
