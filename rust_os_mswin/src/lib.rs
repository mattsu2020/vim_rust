#![cfg(windows)]

use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{env, fs};

use windows::core::PCSTR;
use windows::Win32::System::Console::SetConsoleTitleA;

static G_HINST: AtomicUsize = AtomicUsize::new(0);

/// Store the instance handle of the executable or DLL.
#[no_mangle]
pub extern "C" fn SaveInst(h_inst: usize) {
    G_HINST.store(h_inst, Ordering::Relaxed);
}

#[no_mangle]
pub extern "C" fn os_startup() {}

#[no_mangle]
pub extern "C" fn os_shutdown() {}

#[no_mangle]
pub extern "C" fn os_mkdir(path: *const c_char) -> i32 {
    let c_str = unsafe { CStr::from_ptr(path) };
    match fs::create_dir(c_str.to_string_lossy().into_owned()) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn os_set_title(title: *const c_char) {
    let c_str = unsafe { CStr::from_ptr(title) };
    let bytes = c_str.to_bytes_with_nul();
    unsafe { SetConsoleTitleA(PCSTR(bytes.as_ptr())) };
}

#[no_mangle]
pub extern "C" fn os_chdir(path: *const c_char) -> i32 {
    let c_str = unsafe { CStr::from_ptr(path) };
    match env::set_current_dir(c_str.to_string_lossy().into_owned()) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn saves_instance() {
        SaveInst(1234);
        assert_eq!(G_HINST.load(Ordering::Relaxed), 1234);
    }

    #[test]
    fn creates_directory() {
        let dir = std::env::temp_dir().join("vim_rust_os_test_win32");
        let _ = fs::remove_dir_all(&dir);
        let c_path = CString::new(dir.to_str().unwrap()).unwrap();
        assert_eq!(os_mkdir(c_path.as_ptr()), 0);
        assert!(dir.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn changes_directory() {
        let orig = env::current_dir().unwrap();
        let dir = env::temp_dir();
        let c_path = CString::new(dir.to_str().unwrap()).unwrap();
        assert_eq!(os_chdir(c_path.as_ptr()), 0);
        assert_eq!(env::current_dir().unwrap(), dir);
        let c_orig = CString::new(orig.to_str().unwrap()).unwrap();
        assert_eq!(os_chdir(c_orig.as_ptr()), 0);
    }
}
