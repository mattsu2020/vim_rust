#![cfg(target_os = "windows")]

use libc::{c_int, _isatty};
use windows::Win32::System::SystemInformation::GetTickCount;

#[no_mangle]
pub extern "C" fn os_mswin_startup() {
    // No special initialization required for this minimal example.
}

#[no_mangle]
pub extern "C" fn os_mswin_shutdown() {
    // No special shutdown tasks for this minimal example.
}

#[no_mangle]
pub extern "C" fn os_mswin_get_tick_count() -> u32 {
    unsafe { GetTickCount() }
}

/// Simple wrapper around the CRT `_isatty` to detect console handles.
#[no_mangle]
pub extern "C" fn os_mswin_isatty(fd: c_int) -> c_int {
    unsafe { _isatty(fd) }
}
