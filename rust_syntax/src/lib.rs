use std::ffi::c_void;

#[no_mangle]
pub extern "C" fn rs_syntax_start(_wp: *mut c_void, _lnum: i64) {
    // Placeholder implementation; real logic lives in the Rust crate.
}

#[no_mangle]
pub extern "C" fn rs_syn_update(_startofline: i32) {
    // Placeholder implementation; real logic lives in the Rust crate.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_functions() {
        unsafe {
            rs_syntax_start(std::ptr::null_mut(), 0);
            rs_syn_update(0);
        }
    }
}
