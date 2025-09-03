use std::os::raw::{c_int, c_void};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WinState {
    pub id: c_int,
    pub width: c_int,
    pub height: c_int,
}

#[no_mangle]
pub extern "C" fn rs_win_new(_ptr: *mut c_void, _width: c_int, _height: c_int) {}

#[no_mangle]
pub extern "C" fn rs_win_update(_ptr: *mut c_void, _width: c_int, _height: c_int) {}

#[no_mangle]
pub extern "C" fn rs_win_free(_ptr: *mut c_void) {}

#[no_mangle]
pub extern "C" fn rs_win_save(_ptr: *mut c_void) -> WinState {
    WinState { id: 0, width: 0, height: 0 }
}

#[no_mangle]
pub extern "C" fn rs_win_restore(_state: WinState) -> *mut c_void {
    std::ptr::null_mut()
}
