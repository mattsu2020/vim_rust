use regex::Regex;
use regex::RegexBuilder;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long, c_void};

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
        // simple compatibility flag for case-insensitive
        builder.case_insensitive(true);
    }
    match builder.build() {
        Ok(re) => Box::into_raw(Box::new(RegProg { regex: re })),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn vim_regfree(prog: *mut RegProg) {
    if !prog.is_null() {
        unsafe {
            drop(Box::from_raw(prog));
        }
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
#[derive(Copy, Clone)]
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
pub extern "C" fn vim_regexec_multi(
    rmp: *mut RegMMMatch,
    _win: *mut c_void,
    buf: *mut c_void,
    lnum: c_long,
    col: c_int,
    timed_out: *mut c_int,
) -> c_long {
    // For now we only support searching within the specified line.
    // "buf" is expected to be a pointer to an array of C string pointers,
    // each representing a line of text.
    if !timed_out.is_null() {
        unsafe {
            *timed_out = 0;
        }
    }
    if rmp.is_null() || buf.is_null() {
        return 0;
    }
    unsafe {
        let lines = buf as *const *const c_char;
        // lnum is 1-based
        let line_ptr = *lines.add((lnum - 1) as usize);
        if line_ptr.is_null() {
            return 0;
        }
        let mut rm = RegMatch {
            regprog: (*rmp).regprog,
            startp: [std::ptr::null(); 10],
            endp: [std::ptr::null(); 10],
            rm_matchcol: 0,
            rm_ic: (*rmp).rmm_ic,
        };
        if regexec_internal(&mut rm, line_ptr, col) == 1 {
            let m = &mut *rmp;
            for i in 0..10 {
                if !rm.startp[i].is_null() && !rm.endp[i].is_null() {
                    let start_col = rm.startp[i].offset_from(line_ptr) as c_int;
                    let end_col = rm.endp[i].offset_from(line_ptr) as c_int;
                    m.startpos[i] = Lpos { lnum, col: start_col };
                    m.endpos[i] = Lpos { lnum, col: end_col };
                } else {
                    m.startpos[i] = Lpos { lnum: 0, col: 0 };
                    m.endpos[i] = Lpos { lnum: 0, col: 0 };
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
    let text_str = unsafe { CStr::from_ptr(text).to_string_lossy() };
    let sub_str = unsafe { CStr::from_ptr(sub).to_string_lossy() };
    let result = prog
        .regex
        .replace_all(&text_str, sub_str.as_ref())
        .into_owned();
    CString::new(result).unwrap().into_raw()
}
