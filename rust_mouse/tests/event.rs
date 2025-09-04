use rust_mouse::{rs_handle_mouse_event, rs_mouse_event_len};
use std::os::raw::{c_int, c_long};
use std::ptr;

#[test]
fn records_mouse_events() {
    assert_eq!(rs_mouse_event_len(), 0);
    rs_handle_mouse_event(
        ptr::null_mut(),
        1 as c_int,
        0 as c_int,
        0 as c_long,
        0 as c_int,
    );
    assert_eq!(rs_mouse_event_len(), 1);
}
