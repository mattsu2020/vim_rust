use std::collections::HashMap;
use std::os::raw::{c_int, c_void};
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WinState {
    pub id: c_int,
    pub width: c_int,
    pub height: c_int,
}

static WINDOWS: Lazy<Mutex<HashMap<usize, WinState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[no_mangle]
pub extern "C" fn rs_win_new(ptr: *mut c_void, width: c_int, height: c_int) {
    let mut windows = WINDOWS.lock().unwrap();
    let id = (windows.len() as c_int) + 1;
    windows.insert(ptr as usize, WinState { id, width, height });
}

#[no_mangle]
pub extern "C" fn rs_win_update(ptr: *mut c_void, width: c_int, height: c_int) {
    let mut windows = WINDOWS.lock().unwrap();
    if let Some(state) = windows.get_mut(&(ptr as usize)) {
        state.width = width;
        state.height = height;
    }
}

#[no_mangle]
pub extern "C" fn rs_win_free(ptr: *mut c_void) {
    let mut windows = WINDOWS.lock().unwrap();
    windows.remove(&(ptr as usize));
}

#[no_mangle]
pub extern "C" fn rs_win_save(ptr: *mut c_void) -> WinState {
    let windows = WINDOWS.lock().unwrap();
    windows
        .get(&(ptr as usize))
        .cloned()
        .unwrap_or(WinState { id: 0, width: 0, height: 0 })
}

#[no_mangle]
pub extern "C" fn rs_win_restore(state: WinState) -> *mut c_void {
    let ptr = Box::into_raw(Box::new(0u8)) as *mut c_void;
    let mut windows = WINDOWS.lock().unwrap();
    windows.insert(ptr as usize, state);
    ptr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_restore_roundtrip() {
        let ptr = Box::into_raw(Box::new(0u8)) as *mut c_void;
        rs_win_new(ptr, 80, 24);
        rs_win_update(ptr, 100, 30);
        let saved = rs_win_save(ptr);
        assert_eq!(saved.width, 100);
        assert_eq!(saved.height, 30);
        rs_win_free(ptr);
        unsafe { drop(Box::from_raw(ptr as *mut u8)); }
        let new_ptr = rs_win_restore(saved);
        let restored = rs_win_save(new_ptr);
        assert_eq!(restored, saved);
        rs_win_free(new_ptr);
        unsafe { drop(Box::from_raw(new_ptr as *mut u8)); }
    }
}
