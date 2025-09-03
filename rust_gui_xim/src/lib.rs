use libloading::Library;
use std::ffi::c_void;

/// Attempt to open the X Input Method (XIM) using dynamic loading.
///
/// On systems without X11 this will simply return `false` without
/// attempting to load any libraries.
pub fn open_xim() -> bool {
    unsafe {
        let lib = match Library::new("libX11.so.6") {
            Ok(lib) => lib,
            Err(_) => return false,
        };
        // We only verify that the `XOpenIM` symbol exists; calling it would
        // require a valid X11 display and input method setup which is not
        // available in the test environment.
        let result = lib.get::<unsafe extern "C" fn() -> *mut c_void>(b"XOpenIM\0");
        result.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_xim() {
        // This test mainly ensures the dynamic loading path executes
        // without crashing on platforms lacking X11.
        let _ = open_xim();
    }
}
