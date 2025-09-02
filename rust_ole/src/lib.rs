use std::os::raw::c_int;

#[cfg(target_os = "windows")]
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

/// Initialize COM for OLE automation on Windows.
#[no_mangle]
pub extern "C" fn vim_ole_init() -> c_int {
    #[cfg(target_os = "windows")]
    unsafe {
        if CoInitializeEx(None, COINIT_MULTITHREADED).is_ok() {
            return 1;
        }
        return 0;
    }
    #[cfg(not(target_os = "windows"))]
    { 0 }
}

/// Uninitialize COM when finished.
#[no_mangle]
pub extern "C" fn vim_ole_uninit() {
    #[cfg(target_os = "windows")]
    unsafe {
        CoUninitialize();
    }
}
