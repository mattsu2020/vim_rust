use std::os::raw::{c_char, c_int};

/// Dummy implementation of the cross-command server interface.
#[no_mangle]
pub extern "C" fn vim_xcmdsrv_send(_cmd: *const c_char) -> c_int {
    // Real implementation would communicate with an external server.
    1
}
