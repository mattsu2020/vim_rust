use libc::{c_int, c_long, c_void};

#[cfg(feature = "gui")]
mod gui;
#[cfg(not(feature = "gui"))]
mod cli;

#[cfg(feature = "gui")]
use gui::handle_mouse_event;
#[cfg(not(feature = "gui"))]
use cli::handle_mouse_event;

#[no_mangle]
pub extern "C" fn rs_handle_mouse_event(
    oap: *mut c_void,
    c: c_int,
    dir: c_int,
    count: c_long,
    fixindent: c_int,
) -> c_int {
    handle_mouse_event(oap, c, dir, count, fixindent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_handles_event() {
        let res = rs_handle_mouse_event(std::ptr::null_mut(), 0, 0, 0, 0);
        assert_eq!(res, 0);
    }
}
