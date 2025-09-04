use std::os::raw::c_int;

#[no_mangle]
pub extern "C" fn rs_winclip_beep() -> c_int {
    #[cfg(windows)]
    unsafe {
        winapi::um::winuser::MessageBeep(0);
    }
    0
}
