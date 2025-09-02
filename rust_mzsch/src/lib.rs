use libloading::Library;
use std::os::raw::c_int;

/// Attempt to load the MzScheme library to verify availability.
#[no_mangle]
pub extern "C" fn vim_mzscheme_init() -> c_int {
    match Library::new("libmzscheme.so") {
        Ok(_) => 1,
        Err(_) => 0,
    }
}
