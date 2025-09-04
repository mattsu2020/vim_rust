use std::ffi::CString;
use std::os::raw::c_char;

fn main() {
    let args: Vec<CString> = std::env::args()
        .map(|s| CString::new(s).expect("CString::new failed"))
        .collect();
    let argv: Vec<*const c_char> = args.iter().map(|s| s.as_ptr()).collect();
    let argc = argv.len() as i32;

    if rust_version::rust_handle_args(argc, argv.as_ptr()) != 0 {
        return;
    }

    std::process::exit(rust_editor::rust_editor_main(argc, argv.as_ptr()));
}
