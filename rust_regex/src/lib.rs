use regex::Regex;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[repr(C)]
pub struct RegProg {
    regex: Regex,
}

#[no_mangle]
pub extern "C" fn vim_regcomp(pattern: *const c_char, _flags: c_int) -> *mut RegProg {
    if pattern.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(pattern) };
    let pattern_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    match Regex::new(pattern_str) {
        Ok(re) => Box::into_raw(Box::new(RegProg { regex: re })),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn vim_regfree(prog: *mut RegProg) {
    if !prog.is_null() {
        unsafe { drop(Box::from_raw(prog)); }
    }
}

#[repr(C)]
pub struct RegMatch {
    pub startp: [*const c_char; 10],
    pub endp: [*const c_char; 10],
}

#[repr(C)]
pub struct SearchStat {
    pub cur: c_int,
    pub cnt: c_int,
    pub exact_match: c_int,
    pub incomplete: c_int,
    pub last_maxcount: c_int,
}

#[no_mangle]
pub extern "C" fn vim_regexec(prog: *mut RegProg, text: *const c_char, matchp: *mut RegMatch) -> c_int {
    if prog.is_null() || text.is_null() {
        return 0;
    }
    let prog = unsafe { &*prog };
    let text_str = unsafe { CStr::from_ptr(text).to_string_lossy().into_owned() };
    match prog.regex.captures(&text_str) {
        Some(caps) => {
            if !matchp.is_null() {
                let m = unsafe { &mut *matchp };
                for i in 0..10 {
                    if let Some(cap) = caps.get(i) {
                        m.startp[i] = text_str.as_ptr().wrapping_add(cap.start()) as *const c_char;
                        m.endp[i] = text_str.as_ptr().wrapping_add(cap.end()) as *const c_char;
                    } else {
                        m.startp[i] = std::ptr::null();
                        m.endp[i] = std::ptr::null();
                    }
                }
            }
            1
        }
        None => 0,
    }
}

#[no_mangle]
pub extern "C" fn vim_regsub(prog: *mut RegProg, text: *const c_char, sub: *const c_char) -> *mut c_char {
    if prog.is_null() || text.is_null() || sub.is_null() {
        return std::ptr::null_mut();
    }
    let prog = unsafe { &*prog };
    let text_str = unsafe { CStr::from_ptr(text).to_string_lossy() };
    let sub_str = unsafe { CStr::from_ptr(sub).to_string_lossy() };
    let result = prog.regex.replace_all(&text_str, sub_str.as_ref()).into_owned();
    CString::new(result).unwrap().into_raw()
}

fn compile_pattern(pattern: &str, magic: bool) -> Result<Regex, regex::Error> {
    let pat = if magic {
        pattern.to_string()
    } else {
        regex::escape(pattern)
    };
    Regex::new(&pat)
}

#[no_mangle]
pub extern "C" fn rust_search_update_stat(
    pat: *const c_char,
    text: *const c_char,
    stat: *mut SearchStat,
) {
    if pat.is_null() || text.is_null() || stat.is_null() {
        return;
    }
    let c_pat = unsafe { CStr::from_ptr(pat) };
    let c_text = unsafe { CStr::from_ptr(text) };
    let pattern = match c_pat.to_str() {
        Ok(p) => p,
        Err(_) => return,
    };
    let text = match c_text.to_str() {
        Ok(t) => t,
        Err(_) => return,
    };
    let re = match Regex::new(pattern) {
        Ok(r) => r,
        Err(_) => return,
    };
    let mut cur: c_int = -1;
    let mut cnt: c_int = 0;
    let mut exact: c_int = 0;
    for (i, m) in re.find_iter(text).enumerate() {
        if i == 0 {
            cur = m.start() as c_int;
            if m.start() == 0 {
                exact = 1;
            }
        }
        cnt = i as c_int + 1;
    }
    unsafe {
        (*stat).cur = cur;
        (*stat).cnt = cnt;
        (*stat).exact_match = exact;
        (*stat).incomplete = 0;
        (*stat).last_maxcount = cnt;
    }
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
