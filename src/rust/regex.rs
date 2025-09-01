use regex::Regex;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// Match the given text against the pattern. Returns 1 on match and 0 otherwise.
#[no_mangle]
pub extern "C" fn nfa_regmatch_rs(pattern: *const c_char, text: *const c_char) -> c_int {
    if pattern.is_null() || text.is_null() {
        return 0;
    }
    let pat = unsafe { CStr::from_ptr(pattern).to_string_lossy().into_owned() };
    let txt = unsafe { CStr::from_ptr(text).to_string_lossy().into_owned() };
    match Regex::new(&pat) {
        Ok(re) => if re.is_match(&txt) { 1 } else { 0 },
        Err(_) => 0,
    }
}
