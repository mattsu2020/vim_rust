use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use once_cell::sync::Lazy;
use std::sync::Mutex;

static BREAKPOINTS: Lazy<Mutex<HashSet<(String, i32)>>> = Lazy::new(|| Mutex::new(HashSet::new()));

fn cstr_to_string(s: *const c_char) -> Option<String> {
    if s.is_null() { None } else { unsafe { CStr::from_ptr(s) }.to_str().ok().map(|s| s.to_string()) }
}

#[no_mangle]
pub extern "C" fn rs_debugger_add_breakpoint(file: *const c_char, line: c_int) -> c_int {
    if let Some(f) = cstr_to_string(file) {
        let mut bp = BREAKPOINTS.lock().unwrap();
        bp.insert((f, line as i32));
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn rs_debugger_remove_breakpoint(file: *const c_char, line: c_int) -> c_int {
    if let Some(f) = cstr_to_string(file) {
        let mut bp = BREAKPOINTS.lock().unwrap();
        bp.remove(&(f, line as i32)) as c_int
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn rs_debugger_check_breakpoint(file: *const c_char, line: c_int) -> c_int {
    if let Some(f) = cstr_to_string(file) {
        let bp = BREAKPOINTS.lock().unwrap();
        if bp.contains(&(f, line as i32)) { 1 } else { 0 }
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn rs_debugger_hit_breakpoint(file: *const c_char, line: c_int) -> c_int {
    if rs_debugger_check_breakpoint(file, line) == 1 {
        unsafe { libc::raise(libc::SIGTRAP) };
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
    fn manage_breakpoints() {
        let file = CString::new("test.rs").unwrap();
        assert_eq!(rs_debugger_add_breakpoint(file.as_ptr(), 10), 1);
        assert_eq!(rs_debugger_check_breakpoint(file.as_ptr(), 10), 1);
        assert_eq!(rs_debugger_remove_breakpoint(file.as_ptr(), 10), 1);
        assert_eq!(rs_debugger_check_breakpoint(file.as_ptr(), 10), 0);
    }
}
