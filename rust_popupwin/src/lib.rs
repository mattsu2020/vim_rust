use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::sync::{Mutex, OnceLock};

struct Popup {
    text: String,
    line: c_int,
    col: c_int,
    visible: bool,
}

static POPUPS: OnceLock<Mutex<HashMap<c_int, Popup>>> = OnceLock::new();
static NEXT_ID: OnceLock<Mutex<c_int>> = OnceLock::new();

fn state() -> &'static Mutex<HashMap<c_int, Popup>> {
    POPUPS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> c_int {
    let mut guard = NEXT_ID.get_or_init(|| Mutex::new(1)).lock().unwrap();
    let id = *guard;
    *guard += 1;
    id
}

#[no_mangle]
pub extern "C" fn popupwin_clear() {
    state().lock().unwrap().clear();
}

#[no_mangle]
pub extern "C" fn popupwin_create(text: *const c_char, line: c_int, col: c_int) -> c_int {
    let txt = unsafe {
        if text.is_null() {
            String::new()
        } else {
            CStr::from_ptr(text).to_string_lossy().into_owned()
        }
    };
    let id = next_id();
    state().lock().unwrap().insert(id, Popup { text: txt, line, col, visible: true });
    id
}

#[no_mangle]
pub extern "C" fn popupwin_close(id: c_int) {
    state().lock().unwrap().remove(&id);
}

#[repr(C)]
pub struct PopupPos {
    pub line: c_int,
    pub col: c_int,
    pub visible: c_int,
}

#[no_mangle]
pub extern "C" fn popupwin_getpos(id: c_int, out: *mut PopupPos) -> c_int {
    if let Some(p) = state().lock().unwrap().get(&id) {
        if !out.is_null() {
            unsafe {
                *out = PopupPos { line: p.line, col: p.col, visible: if p.visible { 1 } else { 0 } };
            }
        }
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn popupwin_move(id: c_int, line: c_int, col: c_int) -> c_int {
    if let Some(p) = state().lock().unwrap().get_mut(&id) {
        p.line = line;
        p.col = col;
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn popupwin_show(id: c_int) -> c_int {
    if let Some(p) = state().lock().unwrap().get_mut(&id) {
        p.visible = true;
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn popupwin_hide(id: c_int) -> c_int {
    if let Some(p) = state().lock().unwrap().get_mut(&id) {
        p.visible = false;
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn create_move_getpos() {
        popupwin_clear();
        let text = CString::new("hello").unwrap();
        let id = popupwin_create(text.as_ptr(), 1, 2);
        let mut pos = PopupPos { line: 0, col: 0, visible: 0 };
        assert_eq!(popupwin_getpos(id, &mut pos), 1);
        assert_eq!((pos.line, pos.col, pos.visible), (1, 2, 1));
        assert_eq!(popupwin_move(id, 3, 4), 1);
        assert_eq!(popupwin_getpos(id, &mut pos), 1);
        assert_eq!((pos.line, pos.col), (3, 4));
        assert_eq!(popupwin_hide(id), 1);
        assert_eq!(popupwin_getpos(id, &mut pos), 1);
        assert_eq!(pos.visible, 0);
        popupwin_close(id);
        assert_eq!(popupwin_getpos(id, &mut pos), 0);
    }
}
