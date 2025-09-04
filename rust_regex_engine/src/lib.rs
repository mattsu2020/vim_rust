use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long, c_void};

use once_cell::sync::Lazy;
use regex::Regex;
use regex::RegexBuilder;
use serde::Deserialize;

#[derive(Deserialize)]
struct RegexConfig {
    max_braces: usize,
    max_states: usize,
    max_submatches: usize,
}

static CONFIG: Lazy<RegexConfig> = Lazy::new(|| {
    let text = std::fs::read_to_string("regex_config.toml").unwrap_or_default();
    toml::from_str(&text).unwrap_or(RegexConfig {
        max_braces: 20,
        max_states: 100000,
        max_submatches: 10,
    })
});

pub fn max_braces() -> usize {
    CONFIG.max_braces
}

pub fn max_states() -> usize {
    CONFIG.max_states
}

pub fn max_submatches() -> usize {
    CONFIG.max_submatches
}

// Flag bit 1 enables case-insensitive matching.

#[repr(C)]
pub struct RegProg {
    regex: Regex,
}

#[no_mangle]
pub extern "C" fn vim_regcomp(pattern: *const c_char, flags: c_int) -> *mut RegProg {
    if pattern.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(pattern) };
    let pattern_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let mut builder = RegexBuilder::new(pattern_str);
    if (flags & 1) != 0 {
        builder.case_insensitive(true);
    }
    let regex = match builder.build() {
        Ok(r) => r,
        Err(_) => return std::ptr::null_mut(),
    };
    Box::into_raw(Box::new(RegProg { regex }))
}

#[no_mangle]
pub extern "C" fn vim_regfree(prog: *mut RegProg) {
    if !prog.is_null() {
        unsafe { drop(Box::from_raw(prog)) };
    }
}

#[repr(C)]
pub struct RegMatch {
    pub regprog: *mut RegProg,
    pub startp: *mut *const c_char,
    pub endp: *mut *const c_char,
    pub len: c_int,
    pub rm_matchcol: c_int,
    pub rm_ic: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Lpos {
    pub lnum: c_long,
    pub col: c_int,
}

#[repr(C)]
pub struct RegMMMatch {
    pub regprog: *mut RegProg,
    pub startpos: *mut Lpos,
    pub endpos: *mut Lpos,
    pub len: c_int,
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
    let line_str = match unsafe { CStr::from_ptr(line).to_str() } {
        Ok(s) => s,
        Err(_) => return 0,
    };
    if (col as usize) > line_str.len() {
        return 0;
    }
    let slice = &line_str[(col as usize)..];
    if let Some(caps) = prog.regex.captures(slice) {
        let m = unsafe { &mut *rmp };
        let rm_len = m.len as usize;
        let startp = unsafe { std::slice::from_raw_parts_mut(m.startp, rm_len) };
        let endp = unsafe { std::slice::from_raw_parts_mut(m.endp, rm_len) };
        for i in 0..rm_len {
            startp[i] = std::ptr::null();
            endp[i] = std::ptr::null();
        }
        let limit = max_submatches().min(rm_len);
        for (i, cap) in caps.iter().enumerate().take(limit) {
            if let Some(mat) = cap {
                startp[i] = unsafe { line.add(col as usize + mat.start()) };
                endp[i] = unsafe { line.add(col as usize + mat.end()) };
            }
        }
        return 1;
    }
    0
}

#[no_mangle]
pub extern "C" fn vim_regexec(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> c_int {
    regexec_internal(rmp, line, col)
}

#[no_mangle]
pub extern "C" fn vim_regexec_nl(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> c_int {
    regexec_internal(rmp, line, col)
}

#[no_mangle]
pub extern "C" fn vim_regexec_multi(
    rmp: *mut RegMMMatch,
    _win: *mut c_void,
    buf: *mut c_void,
    lnum: c_long,
    col: c_int,
    timed_out: *mut c_int,
) -> c_long {
    if !timed_out.is_null() {
        unsafe { *timed_out = 0 };
    }
    if rmp.is_null() || buf.is_null() {
        return 0;
    }
    unsafe {
        let lines = buf as *const *const c_char;
        let line_ptr = *lines.add((lnum - 1) as usize);
        if line_ptr.is_null() {
            return 0;
        }
        let len = (*rmp).len as usize;
        let mut startp = vec![std::ptr::null(); len];
        let mut endp = vec![std::ptr::null(); len];
        let mut rm = RegMatch {
            regprog: (*rmp).regprog,
            startp: startp.as_mut_ptr(),
            endp: endp.as_mut_ptr(),
            len: len as c_int,
            rm_matchcol: 0,
            rm_ic: (*rmp).rmm_ic,
        };
        if regexec_internal(&mut rm, line_ptr, col) == 1 {
            let m = &mut *rmp;
            let startpos = std::slice::from_raw_parts_mut(m.startpos, len);
            let endpos = std::slice::from_raw_parts_mut(m.endpos, len);
            for i in 0..len {
                startpos[i] = Lpos { lnum: 0, col: 0 };
                endpos[i] = Lpos { lnum: 0, col: 0 };
            }
            for i in 0..len {
                if !startp[i].is_null() {
                    startpos[i] = Lpos {
                        lnum,
                        col: startp[i].offset_from(line_ptr) as c_int,
                    };
                }
                if !endp[i].is_null() {
                    endpos[i] = Lpos {
                        lnum,
                        col: endp[i].offset_from(line_ptr) as c_int,
                    };
                }
            }
            m.rmm_matchcol = col;
            return lnum;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn vim_regsub(
    prog: *mut RegProg,
    text: *const c_char,
    sub: *const c_char,
) -> *mut c_char {
    if prog.is_null() || text.is_null() || sub.is_null() {
        return std::ptr::null_mut();
    }
    let prog = unsafe { &*prog };
    let text_str = match unsafe { CStr::from_ptr(text).to_str() } {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let sub_str = match unsafe { CStr::from_ptr(sub).to_str() } {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let result = prog.regex.replace(text_str, sub_str);
    CString::new(result.into_owned()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn vim_regex_max_braces() -> c_int {
    max_braces() as c_int
}

#[no_mangle]
pub extern "C" fn vim_regex_max_states() -> c_int {
    max_states() as c_int
}
