use libloading::Library;
use std::os::raw::c_int;

fn init() -> Result<(), ()> {
    unsafe { Library::new("libmzscheme.so") }.map(|_| ()).map_err(|_| ())
}

/// Attempt to load the MzScheme library to verify availability.
#[no_mangle]
pub extern "C" fn vim_mzscheme_init() -> c_int {
    init().map_or(0, |_| 1)
}

/// Alias exposed for compatibility with existing Vim commands.
#[no_mangle]
pub extern "C" fn mzscheme_init() -> c_int {
    vim_mzscheme_init()
}
