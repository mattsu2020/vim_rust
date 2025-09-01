use regex::Regex;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_long};
use std::time::Duration;
use std::thread;
use std::sync::mpsc;

fn compile_pattern(pattern: &str, magic: bool) -> Result<Regex, regex::Error> {
    let pat = if magic {
        pattern.to_string()
    } else {
        regex::escape(pattern)
    };
    Regex::new(&pat)
}

#[no_mangle]
pub extern "C" fn rust_regex_match(
    pat: *const c_char,
    text: *const c_char,
    magic: c_int,
    timeout_ms: c_long,
) -> c_int {
    if pat.is_null() || text.is_null() {
        return 0;
    }
    let c_pat = unsafe { CStr::from_ptr(pat) };
    let c_text = unsafe { CStr::from_ptr(text) };
    let pattern = match c_pat.to_str() {
        Ok(p) => p,
        Err(_) => return 0,
    };
    let text = match c_text.to_str() {
        Ok(t) => t,
        Err(_) => return 0,
    };

    let (tx, rx) = mpsc::channel();
    let pattern = pattern.to_string();
    let text = text.to_string();
    let magic = magic != 0;
    thread::spawn(move || {
        let result = compile_pattern(&pattern, magic)
            .map(|re| re.is_match(&text))
            .unwrap_or(false);
        let _ = tx.send(result);
    });
    let timeout = Duration::from_millis(timeout_ms as u64);
    match rx.recv_timeout(timeout) {
        Ok(v) => if v { 1 } else { 0 },
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::rust_regex_match;
    use std::ffi::CString;

    fn call(pattern: &str, text: &str, magic: bool, timeout: i64) -> bool {
        let pat = CString::new(pattern).unwrap();
        let txt = CString::new(text).unwrap();
        rust_regex_match(pat.as_ptr(), txt.as_ptr(), magic as i32, timeout as i64) != 0
    }

    #[test]
    fn magic_characters() {
        assert!(call("a.c", "abc", true, 1000));
        assert!(!call("a.c", "abc", false, 1000));
    }

    #[test]
    fn timeout_expires() {
        // Zero timeout should cause the matcher to give up before finishing
        assert!(!call("a", "a", true, 0));
    }
}
