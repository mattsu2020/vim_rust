use regex::Regex;
use libc::{c_char, c_int};
use std::ffi::CStr;
use std::ptr;

const NSUBEXP: usize = 10;

#[repr(C)]
pub struct regprog_T {
    regex: *mut Regex,
}

#[repr(C)]
pub struct regmatch_T {
    pub regprog: *mut regprog_T,
    pub startp: [*const c_char; NSUBEXP],
    pub endp: [*const c_char; NSUBEXP],
    pub rm_matchcol: u32,
    pub rm_ic: c_int,
}

#[no_mangle]
pub extern "C" fn vim_regcomp_rs(pattern: *const c_char, _flags: c_int) -> *mut regprog_T {
    if pattern.is_null() {
        return ptr::null_mut();
    }
    let cstr = unsafe { CStr::from_ptr(pattern) };
    let pat = match cstr.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let regex = match Regex::new(pat) {
        Ok(r) => Box::new(r),
        Err(_) => return ptr::null_mut(),
    };
    let prog = Box::new(regprog_T { regex: Box::into_raw(regex) });
    Box::into_raw(prog)
}

#[no_mangle]
pub extern "C" fn vim_regfree_rs(prog: *mut regprog_T) {
    if prog.is_null() {
        return;
    }
    unsafe {
        let prog = Box::from_raw(prog);
        if !prog.regex.is_null() {
            let _ = Box::from_raw(prog.regex);
        }
    }
}

#[no_mangle]
pub extern "C" fn vim_regexec_rs(rmp: *mut regmatch_T, line: *const c_char, col: u32) -> c_int {
    if rmp.is_null() || line.is_null() {
        return 0;
    }
    unsafe {
        let prog = (*rmp).regprog;
        if prog.is_null() || (*prog).regex.is_null() {
            return 0;
        }
        let regex = &*((*prog).regex);
        let cstr = CStr::from_ptr(line);
        if let Ok(s) = cstr.to_str() {
            if let Some(m) = regex.find(&s[col as usize..]) {
                let start = m.start() + col as usize;
                let end = m.end() + col as usize;
                (*rmp).startp[0] = s.as_ptr().add(start) as *const c_char;
                (*rmp).endp[0] = s.as_ptr().add(end) as *const c_char;
                return 1;
            }
        }
    }
    0
}
