use std::collections::HashMap;
use std::os::raw::{c_int, c_void};
use std::sync::{atomic::{AtomicI32, Ordering}, Mutex};
use once_cell::sync::Lazy;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WinState {
    pub id: c_int,
    pub width: c_int,
    pub height: c_int,
}

static NEXT_ID: AtomicI32 = AtomicI32::new(1);
static WINDOWS: Lazy<Mutex<HashMap<usize, WinState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[no_mangle]
pub extern "C" fn rs_win_new(ptr: *mut c_void, width: c_int, height: c_int) {
    if ptr.is_null() {
        return;
    }
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let mut windows = WINDOWS.lock().unwrap();
    windows.insert(ptr as usize, WinState { id, width, height });
}

#[no_mangle]
pub extern "C" fn rs_win_update(ptr: *mut c_void, width: c_int, height: c_int) {
    if ptr.is_null() {
        return;
    }
    let mut windows = WINDOWS.lock().unwrap();
    let entry = windows.entry(ptr as usize).or_insert_with(|| WinState {
        id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
        width,
        height,
    });
    entry.width = width;
    entry.height = height;
}

#[no_mangle]
pub extern "C" fn rs_win_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
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

    #[test]
    fn complex_operations() {
        let w1 = Box::into_raw(Box::new(0u8)) as *mut c_void;
        rs_win_new(w1, 80, 24);

        // simulate scrolling reducing available height and restoring it
        rs_win_update(w1, 80, 20);
        assert_eq!(rs_win_save(w1).height, 20);
        rs_win_update(w1, 80, 24);

        // split the window creating a second one
        let w2 = Box::into_raw(Box::new(0u8)) as *mut c_void;
        rs_win_new(w2, 80, 12);
        rs_win_update(w1, 80, 12);
        {
            let windows = WINDOWS.lock().unwrap();
            assert_eq!(windows.len(), 2);
        }

        // close the split window and resize the first back
        rs_win_free(w2);
        unsafe { drop(Box::from_raw(w2 as *mut u8)); }
        rs_win_update(w1, 80, 24);
        {
            let windows = WINDOWS.lock().unwrap();
            assert_eq!(windows.len(), 1);
            let state = windows.get(&(w1 as usize)).unwrap();
            assert_eq!(state.height, 24);
        }

        rs_win_free(w1);
        unsafe { drop(Box::from_raw(w1 as *mut u8)); }
        assert_eq!(WINDOWS.lock().unwrap().len(), 0);
    }
}
