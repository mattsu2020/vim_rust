use libc::{c_char, c_int, c_long};
use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::sync::Mutex;

#[derive(Default)]
struct HighlightState {
    groups: Vec<(i32, String)>,
    marks: Vec<c_long>,
    matches: Vec<String>,
}

static STATE: Lazy<Mutex<HighlightState>> = Lazy::new(|| Mutex::new(HighlightState::default()));

pub fn register_rule(id: i32, pattern: &str) {
    let mut state = STATE.lock().unwrap();
    state.groups.push((id, pattern.to_string()));
}

pub fn record_match(pattern: &str) {
    STATE.lock().unwrap().matches.push(pattern.to_string());
}

pub fn set_mark(pos: c_long) {
    STATE.lock().unwrap().marks.push(pos);
}

#[no_mangle]
pub extern "C" fn rs_add_highlight(id: c_int, name: *const c_char) {
    if name.is_null() {
        return;
    }
    let cstr = unsafe { CStr::from_ptr(name) };
    if let Ok(name) = cstr.to_str() {
        register_rule(id as i32, name);
    }
}

#[no_mangle]
pub extern "C" fn rs_set_mark(pos: c_long) {
    set_mark(pos);
}

#[no_mangle]
pub extern "C" fn rs_add_match(pattern: *const c_char) -> c_int {
    if pattern.is_null() {
        return 0;
    }
    let cstr = unsafe { CStr::from_ptr(pattern) };
    if let Ok(pat) = cstr.to_str() {
        record_match(pat);
        1
    } else {
        0
    }
}
