use std::ffi::CString;

use rust_drawline;
use rust_screen;

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

    // Demonstrate drawing a line using the Rust implementation.
    let sb = rust_screen::rs_screen_new(40, 1);
    let line = CString::new("Hello from rust_drawline").unwrap();
    rust_drawline::rs_draw_line(sb, 0, line.as_ptr());
    rust_screen::rs_screen_flush(sb, None);
    rust_screen::rs_screen_free(sb);

    println!("Initialization complete");
}
