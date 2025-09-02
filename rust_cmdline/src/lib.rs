use once_cell::sync::Lazy;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::Mutex;

const HIST_CMD: usize = 0;
const HIST_SEARCH: usize = 1;
const HIST_EXPR: usize = 2;
const HIST_INPUT: usize = 3;
const HIST_DEBUG: usize = 4;
const HIST_COUNT: usize = 5;

static HISTORIES: Lazy<Mutex<Vec<Vec<CString>>>> = Lazy::new(|| {
    let mut v = Vec::with_capacity(HIST_COUNT);
    for _ in 0..HIST_COUNT {
        v.push(Vec::new());
    }
    Mutex::new(v)
});

#[no_mangle]
pub extern "C" fn hist_char2type_rs(c: c_int) -> c_int {
    match c as u8 as char {
        ':' => HIST_CMD as c_int,
        '=' => HIST_EXPR as c_int,
        '@' => HIST_INPUT as c_int,
        '>' => HIST_DEBUG as c_int,
        _ => HIST_SEARCH as c_int,
    }
}

#[no_mangle]
pub extern "C" fn history_add_rs(hist_type: c_int, line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }
    let hist_type = hist_type as usize;
    if hist_type >= HIST_COUNT {
        return 0;
    }
    let cstr = unsafe { CStr::from_ptr(line) };
    if let Ok(s) = cstr.to_str() {
        let mut hists = HISTORIES.lock().unwrap();
        hists[hist_type].insert(0, CString::new(s).unwrap());
        if hists[hist_type].len() > 100 {
            hists[hist_type].pop();
        }
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn history_len_rs(hist_type: c_int) -> c_int {
    let hist_type = hist_type as usize;
    if hist_type >= HIST_COUNT {
        return 0;
    }
    let hists = HISTORIES.lock().unwrap();
    hists[hist_type].len() as c_int
}

#[no_mangle]
pub extern "C" fn history_get_rs(hist_type: c_int, idx: c_int) -> *const c_char {
    let hist_type = hist_type as usize;
    if hist_type >= HIST_COUNT {
        return std::ptr::null();
    }
    let hists = HISTORIES.lock().unwrap();
    match hists[hist_type].get(idx as usize) {
        Some(s) => s.as_ptr(),
        None => std::ptr::null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CString, CStr};

    #[test]
    fn test_hist_char2type() {
        assert_eq!(hist_char2type_rs(':' as c_int), HIST_CMD as c_int);
        assert_eq!(hist_char2type_rs('=' as c_int), HIST_EXPR as c_int);
        assert_eq!(hist_char2type_rs('@' as c_int), HIST_INPUT as c_int);
        assert_eq!(hist_char2type_rs('>' as c_int), HIST_DEBUG as c_int);
        assert_eq!(hist_char2type_rs('/' as c_int), HIST_SEARCH as c_int);
    }

    #[test]
    fn test_history_add_and_get() {
        let line = CString::new("hello").unwrap();
        assert_eq!(history_add_rs(HIST_CMD as c_int, line.as_ptr()), 1);
        assert_eq!(history_len_rs(HIST_CMD as c_int), 1);
        let ptr = history_get_rs(HIST_CMD as c_int, 0);
        assert!(!ptr.is_null());
        let s = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap();
        assert_eq!(s, "hello");
    }
}
