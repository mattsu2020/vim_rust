use once_cell::sync::Lazy;
use rust_typval::TypVal;
use std::collections::HashMap;
use std::sync::Mutex;

static VIM_VARS: Lazy<Mutex<HashMap<i32, TypVal>>> = Lazy::new(|| Mutex::new(HashMap::new()));

static WINDOWS: Lazy<Mutex<Vec<i32>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Store a numeric Vim variable identified by `idx`.
pub fn set_vim_var_nr(idx: i32, val: i64) {
    let mut vars = VIM_VARS.lock().unwrap();
    vars.insert(idx, TypVal::Number(val));
}

/// Retrieve a previously stored numeric Vim variable.
pub fn get_vim_var_nr(idx: i32) -> Option<i64> {
    match VIM_VARS.lock().unwrap().get(&idx) {
        Some(TypVal::Number(n)) => Some(*n),
        _ => None,
    }
}

/// Store a string Vim variable identified by `idx`.
pub fn set_vim_var_str(idx: i32, val: &str) {
    let mut vars = VIM_VARS.lock().unwrap();
    vars.insert(idx, TypVal::String(val.to_string()));
}

/// Retrieve a previously stored string Vim variable.
pub fn get_vim_var_str(idx: i32) -> Option<String> {
    match VIM_VARS.lock().unwrap().get(&idx) {
        Some(TypVal::String(s)) => Some(s.clone()),
        _ => None,
    }
}

/// Create a new window and return its id.  The first window gets id 1.
pub fn win_create() -> i32 {
    let mut wins = WINDOWS.lock().unwrap();
    let id = (wins.len() as i32) + 1;
    wins.push(id);
    id
}

/// Return the id of window `winnr`.  If `winnr` is 0 or negative, the current
/// window (the first one) is used.
pub fn win_getid(winnr: i32) -> i32 {
    let wins = WINDOWS.lock().unwrap();
    if winnr <= 0 {
        return *wins.first().unwrap_or(&0);
    }
    wins.get((winnr - 1) as usize).copied().unwrap_or(0)
}
