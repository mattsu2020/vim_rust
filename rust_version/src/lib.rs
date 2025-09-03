use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn vim_version() -> *const c_char {
    static VERSION: &[u8] = b"Rust Vim 0.1\0";
    VERSION.as_ptr() as *const c_char
}
