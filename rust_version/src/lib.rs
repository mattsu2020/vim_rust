use once_cell::sync::Lazy;
use std::ffi::CString;
use std::os::raw::c_char;

static SHORT_VERSION: Lazy<CString> = Lazy::new(|| {
    let ver = env!("CARGO_PKG_VERSION");
    let git = env!("GIT_HASH");
    CString::new(format!("{} ({})", ver, git)).unwrap()
});

static LONG_VERSION: Lazy<CString> = Lazy::new(|| {
    let ver = env!("CARGO_PKG_VERSION");
    let date = env!("BUILD_DATE");
    CString::new(format!("Vim {} built {}", ver, date)).unwrap()
});

#[no_mangle]
pub extern "C" fn rust_short_version() -> *const c_char {
    SHORT_VERSION.as_ptr()
}

#[no_mangle]
pub extern "C" fn rust_long_version() -> *const c_char {
    LONG_VERSION.as_ptr()
}
