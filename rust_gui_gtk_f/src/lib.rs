use libloading::Library;
use std::ffi::{c_char, c_int};

/// Initialize GTK using a dynamically loaded `gtk_init_check` call.
///
/// Passing null pointers for argc/argv matches the C API's allowance for
/// optional arguments.  Returns `true` on success and `false` otherwise.
pub fn gtk_init_check() -> bool {
    unsafe {
        let lib = match Library::new("libgtk-3.so.0") {
            Ok(lib) => lib,
            Err(_) => return false,
        };
        let func: libloading::Symbol<
            unsafe extern "C" fn(*mut c_int, *mut *mut *mut c_char) -> i32,
        > = match lib.get(b"gtk_init_check\0") {
            Ok(f) => f,
            Err(_) => return false,
        };
        // Call `gtk_init_check(NULL, NULL)`; non-zero indicates success.
        func(std::ptr::null_mut(), std::ptr::null_mut()) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_gtk() {
        // Should not panic even if GTK is unavailable.
        let _ = gtk_init_check();
    }
}
