use std::ffi::CString;
use std::os::raw::c_char;

fn main() {
    let args: Vec<CString> = std::env::args()
        .map(|a| CString::new(a).expect("CString::new failed"))
        .collect();
    let c_args: Vec<*const c_char> = args.iter().map(|s| s.as_ptr()).collect();
    let argc = c_args.len() as i32;
    let argv = c_args.as_ptr();

    let handled = rust_version::rust_handle_args(argc, argv);
    if handled != 0 {
        return;
    }

    rust_editor::rust_editor_main(argc, argv);
}
