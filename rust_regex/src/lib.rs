use regex::Regex;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long, c_void};

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
    pub regprog: *mut RegProg,
    pub startp: [*const c_char; 10],
    pub endp: [*const c_char; 10],
    pub rm_matchcol: c_int,
    pub rm_ic: c_int,
}

#[repr(C)]
pub struct Lpos {
    pub lnum: c_long,
    pub col: c_int,
}

#[repr(C)]
pub struct RegMMMatch {
    pub regprog: *mut RegProg,
    pub startpos: [Lpos; 10],
    pub endpos: [Lpos; 10],
    pub rmm_matchcol: c_int,
    pub rmm_ic: c_int,
    pub rmm_maxcol: c_int,
}

fn regexec_internal(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> c_int {
    if rmp.is_null() || line.is_null() {
        return 0;
    }
    let prog_ptr = unsafe { (*rmp).regprog };
    if prog_ptr.is_null() {
        return 0;
    }
    let prog = unsafe { &*prog_ptr };
    let line_bytes = unsafe { CStr::from_ptr(line).to_bytes() };
    if (col as usize) > line_bytes.len() {
        return 0;
    }
    let slice_bytes = &line_bytes[(col as usize)..];
    let slice_str = match std::str::from_utf8(slice_bytes) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    match prog.regex.captures(slice_str) {
        Some(caps) => {
            let m = unsafe { &mut *rmp };
            for i in 0..10 {
                if let Some(cap) = caps.get(i) {
                    let start = col as usize + cap.start();
                    let end = col as usize + cap.end();
                    m.startp[i] = unsafe { line.add(start) };
                    m.endp[i] = unsafe { line.add(end) };
                } else {
                    m.startp[i] = std::ptr::null();
                    m.endp[i] = std::ptr::null();
                }
            }
            1
        }
        None => 0,
    }
}

#[no_mangle]
pub extern "C" fn vim_regexec(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> c_int {
    regexec_internal(rmp, line, col)
}

#[no_mangle]
pub extern "C" fn vim_regexec_nl(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> c_int {
    // For simplicity the implementation is the same as vim_regexec.
    regexec_internal(rmp, line, col)
}

#[no_mangle]
pub extern "C" fn vim_regexec_multi(_rmp: *mut RegMMMatch, _win: *mut c_void,
                                     _buf: *mut c_void, _lnum: c_long, _col: c_int,
                                     _timed_out: *mut c_int) -> c_long {
    0
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
