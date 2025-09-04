use std::ffi::CString;

#[no_mangle]
pub extern "C" fn vim_entry_main() {
    // Early init placeholders
    // println!("[rust_entry] early init");

    // Example: call into Rust bufwrite helper (dummy) to show cross-crate call
    let path = CString::new("startup.log").expect("CString::new failed");
    unsafe {
        extern "C" { fn bufwrite_dummy(path: *const i8); }
        bufwrite_dummy(path.as_ptr());
    }

    // Example: call into Vim9class crate demo function
    let class_src = CString::new("class Demo").expect("CString::new failed");
    let _len = unsafe {
        extern "C" { fn rs_vim9class_eval(src: *const i8) -> i32; }
        rs_vim9class_eval(class_src.as_ptr())
    };
}

