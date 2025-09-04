use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

static VERSION_TEXT: &str = concat!(
    "VIM - Vi IMproved (rust stub)\n",
    "Minimal build: features=tiny (rust)\n",
    "Compiled by rust-version\n",
    "Included patches: none\n",
);

#[no_mangle]
pub extern "C" fn rust_handle_args(argc: c_int, argv: *const *const c_char) -> c_int {
    if argc <= 1 || argv.is_null() {
        return 0;
    }
    // Safety: caller guarantees argv is a valid C argv array of length argc.
    for i in 1..(argc as isize) {
        let p = unsafe { *argv.offset(i) };
        if p.is_null() { continue; }
        let s = unsafe { CStr::from_ptr(p) };
        if let Ok(opt) = s.to_str() {
            match opt {
                "--version" | "-v" => {
                    print!("{}", VERSION_TEXT);
                    return 1;
                }
                "--help" | "-h" => {
                    println!("Usage: vim [--version]");
                    return 1;
                }
                _ => {}
            }
        }
    }
    0
}

