use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

fn main() {
    // Collect command line arguments and convert to C strings.
    let args: Vec<CString> = std::env::args()
        .map(|arg| CString::new(arg).expect("CString::new failed"))
        .collect();

    // Build argv array with a trailing null pointer.
    let mut c_args: Vec<*const c_char> = args.iter().map(|a| a.as_ptr()).collect();
    c_args.push(ptr::null());

    let argc = (c_args.len() - 1) as i32;
    let argv = c_args.as_ptr();

    // Handle --help/--version early exits.
    if rust_version::rust_handle_args(argc, argv) != 0 {
        return;
    }

    // Launch the editor and exit with its return code.
    std::process::exit(rust_editor::rust_editor_main(argc, argv));
}
