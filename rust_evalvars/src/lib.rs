use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_longlong};
use std::sync::Mutex;

#[derive(Clone, Debug)]
enum Value {
    Number(i64),
    String(String),
}

static VIM_VARS: Lazy<Mutex<HashMap<i32, Value>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static WINDOWS: Lazy<Mutex<Vec<i32>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

#[no_mangle]
pub extern "C" fn rs_set_vim_var_nr(idx: c_int, val: c_longlong) {
    let mut vars = VIM_VARS.lock().unwrap();
    vars.insert(idx, Value::Number(val));
}

#[no_mangle]
pub extern "C" fn rs_get_vim_var_nr(idx: c_int, out: *mut c_longlong) -> bool {
    if out.is_null() {
        return false;
    }
    if let Some(Value::Number(n)) = VIM_VARS.lock().unwrap().get(&idx) {
        unsafe { *out = *n; }
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn rs_set_vim_var_str(idx: c_int, val: *const c_char) {
    if val.is_null() {
        return;
    }
    let s = unsafe { CStr::from_ptr(val) };
    if let Ok(text) = s.to_str() {
        let mut vars = VIM_VARS.lock().unwrap();
        vars.insert(idx, Value::String(text.to_string()));
    }
}

#[no_mangle]
pub extern "C" fn rs_eval_and(a: c_longlong, b: c_longlong) -> c_longlong {
    a & b
}

#[no_mangle]
pub extern "C" fn rs_win_create() -> c_int {
    let mut wins = WINDOWS.lock().unwrap();
    let id = (wins.len() as i32) + 1;
    wins.push(id);
    id
}

#[no_mangle]
pub extern "C" fn rs_win_getid(winnr: c_int) -> c_int {
    let wins = WINDOWS.lock().unwrap();
    if winnr <= 0 {
        return *wins.first().unwrap_or(&0);
    }
    wins.get((winnr - 1) as usize).copied().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn var_numbers_roundtrip() {
        let mut out: c_longlong = 0;
        assert!(!rs_get_vim_var_nr(1, &mut out));
        rs_set_vim_var_nr(1, 42);
        assert!(rs_get_vim_var_nr(1, &mut out));
        assert_eq!(out, 42);
    }

    #[test]
    fn and_operation() {
        assert_eq!(rs_eval_and(6, 3), 2);
    }

    #[test]
    fn window_ids() {
        let id1 = rs_win_create();
        let id2 = rs_win_create();
        assert_eq!(rs_win_getid(0), id1);
        assert_eq!(rs_win_getid(2), id2);
        assert_eq!(rs_win_getid(3), 0);
    }
}

