use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::{Mutex, OnceLock};

use rust_popupwin::{popupwin_clear, popupwin_create};

#[repr(C)]
pub struct PumItem {
    pub pum_text: *const c_char,
    pub pum_kind: *const c_char,
    pub pum_extra: *const c_char,
    pub pum_info: *const c_char,
    pub pum_cpt_source_idx: c_int,
    pub pum_user_abbr_hlattr: c_int,
    pub pum_user_kind_hlattr: c_int,
}

struct MenuState {
    items: Vec<String>,
    popups: Vec<c_int>,
    selected: c_int,
}

static STATE: OnceLock<Mutex<MenuState>> = OnceLock::new();

fn state() -> &'static Mutex<MenuState> {
    STATE.get_or_init(|| {
        Mutex::new(MenuState {
            items: Vec::new(),
            popups: Vec::new(),
            selected: -1,
        })
    })
}

fn to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
    }
}

#[no_mangle]
pub extern "C" fn pum_display(array: *const PumItem, size: c_int, selected: c_int) {
    let mut st = state().lock().unwrap();
    st.items.clear();
    st.popups.clear();
    popupwin_clear();
    if array.is_null() || size <= 0 {
        st.selected = -1;
        return;
    }
    let slice = unsafe { std::slice::from_raw_parts(array, size as usize) };
    for (i, it) in slice.iter().enumerate() {
        let text = to_string(it.pum_text);
        let id = popupwin_create(it.pum_text, (i + 1) as c_int, 0);
        st.items.push(text);
        st.popups.push(id);
    }
    st.selected = if selected >= 0 && (selected as usize) < st.items.len() {
        selected
    } else {
        -1
    };
}

#[no_mangle]
pub extern "C" fn pum_call_update_screen() {}

#[no_mangle]
pub extern "C" fn pum_under_menu(_row: c_int, _col: c_int, _only_redrawing: c_int) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn pum_redraw() {}

#[no_mangle]
pub extern "C" fn pum_position_info_popup(_wp: *mut c_void) {}

#[no_mangle]
pub extern "C" fn pum_undisplay() {
    let mut st = state().lock().unwrap();
    st.items.clear();
    st.popups.clear();
    st.selected = -1;
    popupwin_clear();
}

#[no_mangle]
pub extern "C" fn pum_clear() {
    pum_undisplay();
}

#[no_mangle]
pub extern "C" fn pum_visible() -> c_int {
    if state().lock().unwrap().items.is_empty() { 0 } else { 1 }
}

#[no_mangle]
pub extern "C" fn pum_redraw_in_same_position() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn pum_may_redraw() {}

#[no_mangle]
pub extern "C" fn pum_get_height() -> c_int {
    state().lock().unwrap().items.len() as c_int
}

#[no_mangle]
pub extern "C" fn pum_set_event_info(_dict: *mut c_void) {}

#[repr(C)]
pub struct PumItemOwned {
    pub pum_text: *mut c_char,
    pub pum_kind: *mut c_char,
    pub pum_extra: *mut c_char,
    pub pum_info: *mut c_char,
    pub pum_cpt_source_idx: c_int,
    pub pum_user_abbr_hlattr: c_int,
    pub pum_user_kind_hlattr: c_int,
}

/// Frees an array of [`PumItemOwned`] previously returned by [`split_message`].
///
/// This reclaims both the individual strings and the array itself.
#[no_mangle]
pub extern "C" fn free_pum_items(ptr: *mut PumItemOwned, count: usize) {
    if ptr.is_null() || count == 0 {
        return;
    }
    unsafe {
        // Recreate the Vec so that Rust can properly deallocate the memory.
        let items = Vec::from_raw_parts(ptr, count, count);
        for item in items {
            if !item.pum_text.is_null() {
                let _ = CString::from_raw(item.pum_text);
            }
            if !item.pum_kind.is_null() {
                let _ = CString::from_raw(item.pum_kind);
            }
            if !item.pum_extra.is_null() {
                let _ = CString::from_raw(item.pum_extra);
            }
            if !item.pum_info.is_null() {
                let _ = CString::from_raw(item.pum_info);
            }
        }
        // `items` drops here and frees the allocation of the array itself.
    }
}

/// A safe wrapper around a raw pointer to [`PumItemOwned`] values.
/// When dropped it automatically releases all allocated strings and
/// the array itself by calling [`free_pum_items`].
pub struct OwnedPumItems {
    ptr: *mut PumItemOwned,
    count: usize,
}

impl OwnedPumItems {
    /// # Safety
    ///
    /// `ptr` must be a pointer returned by [`split_message`] and `count`
    /// must be the number of items returned.
    pub unsafe fn from_raw(ptr: *mut PumItemOwned, count: usize) -> Self {
        Self { ptr, count }
    }

    pub fn as_ptr(&self) -> *const PumItemOwned {
        self.ptr
    }

    pub fn len(&self) -> usize {
        self.count
    }
}

impl Drop for OwnedPumItems {
    fn drop(&mut self) {
        free_pum_items(self.ptr, self.count)
    }
}

#[no_mangle]
pub extern "C" fn split_message(mesg: *mut c_char, array: *mut *mut PumItemOwned) -> c_int {
    if mesg.is_null() || array.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(mesg) }.to_string_lossy().into_owned();
    let lines: Vec<&str> = s.lines().collect();
    let mut items: Vec<PumItemOwned> = Vec::with_capacity(lines.len());
    for line in lines.iter() {
        let cstr = CString::new(*line).unwrap();
        let ptr = cstr.into_raw();
        items.push(PumItemOwned {
            pum_text: ptr,
            pum_kind: std::ptr::null_mut(),
            pum_extra: std::ptr::null_mut(),
            pum_info: std::ptr::null_mut(),
            pum_cpt_source_idx: 0,
            pum_user_abbr_hlattr: 0,
            pum_user_kind_hlattr: 0,
        });
    }
    let ptr = items.as_mut_ptr();
    std::mem::forget(items);
    unsafe { *array = ptr; }
    lines.len() as c_int
}

#[no_mangle]
pub extern "C" fn ui_remove_balloon() {}

#[no_mangle]
pub extern "C" fn ui_post_balloon(_mesg: *mut c_char, _list: *mut c_void) {}

#[no_mangle]
pub extern "C" fn ui_may_remove_balloon() {}

#[no_mangle]
pub extern "C" fn pum_show_popupmenu(_menu: *mut c_void) {}

#[no_mangle]
pub extern "C" fn pum_make_popup(_path_name: *mut c_char, _use_mouse_pos: c_int) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn display_and_clear() {
        pum_clear();
        let t1 = CString::new("first").unwrap();
        let t2 = CString::new("second").unwrap();
        let items = [
            PumItem { pum_text: t1.as_ptr(), pum_kind: std::ptr::null(), pum_extra: std::ptr::null(), pum_info: std::ptr::null(), pum_cpt_source_idx: 0, pum_user_abbr_hlattr: 0, pum_user_kind_hlattr: 0 },
            PumItem { pum_text: t2.as_ptr(), pum_kind: std::ptr::null(), pum_extra: std::ptr::null(), pum_info: std::ptr::null(), pum_cpt_source_idx: 0, pum_user_abbr_hlattr: 0, pum_user_kind_hlattr: 0 },
        ];
        pum_display(items.as_ptr(), items.len() as c_int, -1);
        assert_eq!(pum_visible(), 1);
        assert_eq!(pum_get_height(), 2);
        pum_clear();
        assert_eq!(pum_visible(), 0);
    }

    #[test]
    fn split_message_creates_items() {
        let msg = CString::new("one\ntwo").unwrap();
        let msg_ptr = msg.into_raw();
        let mut out: *mut PumItemOwned = std::ptr::null_mut();
        let n = split_message(msg_ptr, &mut out);
        assert_eq!(n, 2);
        unsafe {
            // Wrap in OwnedPumItems to ensure cleanup.
            let _items = OwnedPumItems::from_raw(out, n as usize);
            // Reclaim the message string allocated with into_raw().
            let _ = CString::from_raw(msg_ptr);
        }
    }
}
