use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long, c_void};

mod class;
mod backtrack;
mod nfa;

enum Engine {
    Backtrack(Vec<u8>),
    NFA(nfa::Prog),
}

pub struct RegProg {
    engine: Engine,
    ic: bool,
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
    let ic = (flags & 1) != 0;
    let use_nfa = (flags & 2) != 0;
    let prog = if use_nfa {
        match nfa::compile(pattern_str) {
            Some(p) => RegProg { engine: Engine::NFA(p), ic },
            None => return std::ptr::null_mut(),
        }
    } else {
        RegProg { engine: Engine::Backtrack(pattern_str.as_bytes().to_vec()), ic }
    };
    Box::into_raw(Box::new(prog))
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

fn run_search(prog: &RegProg, text: &[u8]) -> Option<(usize, usize)> {
    match &prog.engine {
        Engine::Backtrack(pat) => backtrack::search(pat, text, prog.ic),
        Engine::NFA(p) => nfa::search(p, text, prog.ic),
    }
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
    let slice = &line_bytes[(col as usize)..];
    if let Some((s, e)) = run_search(prog, slice) {
        let m = unsafe { &mut *rmp };
        for i in 0..10 {
            m.startp[i] = std::ptr::null();
            m.endp[i] = std::ptr::null();
        }
        unsafe {
            m.startp[0] = line.add(col as usize + s);
            m.endp[0] = line.add(col as usize + e);
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
                m.startpos[i] = Lpos { lnum: 0, col: 0 };
                m.endpos[i] = Lpos { lnum: 0, col: 0 };
            }
            m.startpos[0] = Lpos {
                lnum,
                col: rm.startp[0].offset_from(line_ptr) as c_int,
            };
            m.endpos[0] = Lpos {
                lnum,
                col: rm.endp[0].offset_from(line_ptr) as c_int,
            };
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
    let text_str = unsafe { CStr::from_ptr(text).to_bytes() };
    let sub_str = unsafe { CStr::from_ptr(sub).to_bytes() };
    if let Some((s, e)) = run_search(prog, text_str) {
        let mut result = Vec::new();
        result.extend_from_slice(&text_str[..s]);
        result.extend_from_slice(sub_str);
        result.extend_from_slice(&text_str[e..]);
        CString::new(result).unwrap().into_raw()
    } else {
        CString::new(text_str).unwrap().into_raw()
    }
}
