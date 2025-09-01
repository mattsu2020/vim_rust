use regex::Regex;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

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
