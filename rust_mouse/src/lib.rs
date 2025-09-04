use libc::{c_int, c_long, c_void};
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::Mutex;

const UPD_NOT_VALID: c_int = 40;

#[cfg(not(test))]
extern "C" {
    fn redraw_win_later(wp: *mut c_void, update: c_int);
    fn pum_visible() -> c_int;
    static mut curwin: *mut c_void;
}

#[cfg(test)]
mod stubs {
    use super::*;
    #[no_mangle]
    pub extern "C" fn redraw_win_later(_wp: *mut c_void, _update: c_int) {}
    #[no_mangle]
    pub extern "C" fn pum_visible() -> c_int {
        0
    }
    #[no_mangle]
    pub static mut curwin: *mut c_void = std::ptr::null_mut();
}

#[cfg(test)]
use stubs::*;

#[cfg(not(feature = "gui"))]
mod cli;
#[cfg(feature = "gui")]
mod gui;

#[cfg(not(feature = "gui"))]
use cli::handle_mouse_event;
#[cfg(feature = "gui")]
use gui::handle_mouse_event;

#[no_mangle]
pub extern "C" fn rs_handle_mouse_event(
    oap: *mut c_void,
    c: c_int,
    dir: c_int,
    count: c_long,
    fixindent: c_int,
) -> c_int {
    EVENTS.lock().unwrap().push_back((c, dir, count, fixindent));
    handle_mouse_event(oap, c, dir, count, fixindent)
}

#[no_mangle]
pub extern "C" fn rs_redraw_pum_overlap() {
    unsafe {
        if pum_visible() != 0 && !curwin.is_null() {
            redraw_win_later(curwin, UPD_NOT_VALID);
        }
    }
}

static EVENTS: Lazy<Mutex<VecDeque<(c_int, c_int, c_long, c_int)>>> =
    Lazy::new(|| Mutex::new(VecDeque::new()));

#[no_mangle]
pub extern "C" fn rs_mouse_event_len() -> usize {
    EVENTS.lock().unwrap().len()
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
