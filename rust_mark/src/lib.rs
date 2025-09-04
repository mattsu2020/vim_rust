use std::collections::HashMap;
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Position {
    pub line: i64,
    pub col: i64,
}

static MARKS: OnceLock<Mutex<HashMap<u8, Position>>> = OnceLock::new();

fn marks() -> &'static Mutex<HashMap<u8, Position>> {
    MARKS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[no_mangle]
pub extern "C" fn mark_set(name: c_char, line: i64, col: i64) {
    let mut map = marks().lock().unwrap();
    map.insert(name as u8, Position { line, col });
}

#[no_mangle]
pub extern "C" fn mark_get(name: c_char, out: *mut Position) -> bool {
    let map = marks().lock().unwrap();
    if let Some(pos) = map.get(&(name as u8)) {
        if !out.is_null() {
            unsafe { *out = *pos; }
        }
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn mark_clear(name: c_char) {
    let mut map = marks().lock().unwrap();
    map.remove(&(name as u8));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_lifecycle() {
        mark_set('a' as c_char, 10, 20);
        let mut pos = Position { line: 0, col: 0 };
        assert!(mark_get('a' as c_char, &mut pos as *mut Position));
        assert_eq!(pos.line, 10);
        assert_eq!(pos.col, 20);
        mark_clear('a' as c_char);
        assert!(!mark_get('a' as c_char, &mut pos as *mut Position));
    }
}
