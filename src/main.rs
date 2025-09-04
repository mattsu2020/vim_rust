use std::ffi::CString;
use std::os::raw::c_char;
use std::process;
use std::ptr;

/// Placeholder for platform specific early initialization.
fn mch_early_init() {
    // In the original C implementation this prepares the OS layer.
    println!("Performing early initialization");
}

/// Placeholder for autocommand setup.
fn autocmd_init() {
    println!("Initializing autocommands");
}

/// Placeholder for common initialization step 1.
fn common_init_1() {
    println!("Common init step 1");
}

/// Placeholder for common initialization step 2.
fn common_init_2() {
    println!("Common init step 2");
}

fn main() {
    mch_early_init();

    // Example call demonstrating interaction with a Rust function originally
    // exposed to C.  In main.c this wrote a log file during startup.
    let path = CString::new("startup.log").expect("CString::new failed");
    rust_bufwrite::bufwrite_dummy(path.as_ptr());

    autocmd_init();
    common_init_1();
    common_init_2();

    // Demonstrate calling into the Vim9 class implementation in Rust.
    let class_src = CString::new("class Demo").expect("CString::new failed");
    let len = rust_vim9class::rs_vim9class_eval(class_src.as_ptr());
    println!("Class name length: {}", len);

    // Collect arguments for passing to the C-style interfaces.
    let args: Vec<String> = std::env::args().collect();
    let c_args: Vec<CString> = args
        .iter()
        .map(|a| CString::new(a.as_str()).expect("CString::new failed"))
        .collect();
    let mut argv: Vec<*const c_char> = c_args.iter().map(|a| a.as_ptr()).collect();
    argv.push(ptr::null());
    let argc = c_args.len() as i32;

    unsafe {
        if rust_version::rust_handle_args(argc, argv.as_ptr()) != 0 {
            return;
        }
        let code = rust_editor::rust_editor_main(argc, argv.as_ptr());
        process::exit(code);
    }
}
