use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::os::raw::{c_char, c_int};
use std::sync::{Mutex, atomic::{AtomicI32, Ordering}};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextPropType {
    pub id: i32,
    pub name: String,
}

static GLOBAL_TYPES: OnceCell<Mutex<HashMap<String, TextPropType>>> = OnceCell::new();
static NEXT_ID: AtomicI32 = AtomicI32::new(1);

fn get_map() -> &'static Mutex<HashMap<String, TextPropType>> {
    GLOBAL_TYPES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn add_prop_type(name: &str) -> TextPropType {
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let tp = TextPropType { id, name: name.to_string() };
    get_map().lock().unwrap().insert(name.to_string(), tp.clone());
    tp
}

pub fn find_prop_type_id(name: &str) -> Option<i32> {
    get_map().lock().unwrap().get(name).map(|tp| tp.id)
}

pub fn clear_all() {
    if let Some(map) = GLOBAL_TYPES.get() {
        map.lock().unwrap().clear();
    }
    NEXT_ID.store(1, Ordering::SeqCst);
}

#[no_mangle]
pub extern "C" fn rust_textprop_add_type(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }
    let cstr = unsafe { std::ffi::CStr::from_ptr(name) };
    match cstr.to_str() {
        Ok(s) => add_prop_type(s).id,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn rust_textprop_find_type_id(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }
    let cstr = unsafe { std::ffi::CStr::from_ptr(name) };
    match cstr.to_str() {
        Ok(s) => find_prop_type_id(s).unwrap_or(0),
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_find_type() {
        clear_all();
        let tp = add_prop_type("comment");
        assert!(tp.id > 0);
        assert_eq!(find_prop_type_id("comment"), Some(tp.id));
        assert_eq!(find_prop_type_id("missing"), None);
    }
}
