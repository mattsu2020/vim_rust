#![cfg(windows)]

use std::os::raw::{c_char, c_int};

extern "C" {
    fn VimMain(argc: c_int, argv: *mut *mut c_char) -> c_int;
    fn SaveInst(h_inst: usize);
}

/// Entry point used by the GUI subsystem on Windows.
#[no_mangle]
pub extern "system" fn wWinMain(
    h_instance: usize,
    _h_prev: usize,
    _cmd_line: *mut u16,
    _cmd_show: c_int,
) -> c_int {
    unsafe {
        SaveInst(h_instance);
        VimMain(0, std::ptr::null_mut())
    }
}

/// Console entry point for Windows.
#[no_mangle]
pub extern "C" fn wmain(
    _argc: c_int,
    _argv: *mut *mut u16,
) -> c_int {
    unsafe {
        SaveInst(0);
        VimMain(0, std::ptr::null_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[no_mangle]
    extern "C" fn VimMain(_argc: c_int, _argv: *mut *mut c_char) -> c_int { 0 }
    #[no_mangle]
    extern "C" fn SaveInst(_h: usize) {}

    #[test]
    fn exercise_entry_points() {
        assert_eq!(wmain(0, std::ptr::null_mut()), 0);
        assert_eq!(wWinMain(1, 0, std::ptr::null_mut(), 0), 0);
    }
}
