//! Lightweight regular expression and related matching utilities used by the
//! Rust port of Vim.  This crate started as a direct translation of the small
//! C regex engine used in the Vim sources (see `regexp.c`, `regexp_bt.c` and
//! `regexp_nfa.c`).  The goal of this crate is not to be feature complete but
//! to offer a reasonably small and safe interface for the parts of the Vim
//! code base that require regular expression functionality.
//!
//! In addition to the traditional regex engine the original C code also
//! exposes fuzzy matching and a simple line based matching algorithm.  These
//! have been reimplemented in safe Rust and are available through the
//! [`fuzzy_match`] and [`line_match`] helpers.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long, c_void};
use regex::bytes::{NoExpand, Regex, RegexBuilder};

mod fuzzy;
mod linematch;

pub use fuzzy::fuzzy_match;
pub use linematch::line_match;

/// Search for a match of `pat` anywhere in `text`.
///
/// This helper mirrors the functionality provided by the old miniature
/// regex engine but is now backed by the `regex` crate.  It is primarily
/// exposed for benchmarks and tests.
pub fn search(pat: &[u8], text: &[u8], ic: bool) -> Option<(usize, usize)> {
    let pat_str = std::str::from_utf8(pat).ok()?;
    let mut builder = RegexBuilder::new(pat_str);
    builder.case_insensitive(ic);
    let re = builder.build().ok()?;
    re.find(text).map(|m| (m.start(), m.end()))
}

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
    match builder.build() {
        Ok(regex) => Box::into_raw(Box::new(RegProg { regex })),
        Err(_) => std::ptr::null_mut(),
    }
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
    if let Some(mat) = prog.regex.find(slice) {
        let rm = unsafe { &mut *rmp };
        for i in 0..10 {
            rm.startp[i] = std::ptr::null();
            rm.endp[i] = std::ptr::null();
        }
        unsafe {
            rm.startp[0] = line.add(col as usize + mat.start());
            rm.endp[0] = line.add(col as usize + mat.end());
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
pub extern "C" fn vim_regexec_prog(
    prog: *mut *mut RegProg,
    ignore_case: c_int,
    line: *const c_char,
    col: c_int,
) -> c_int {
    if prog.is_null() {
        return 0;
    }
    let regprog = unsafe { *prog };
    if regprog.is_null() {
        return 0;
    }
    let mut rmp = RegMatch {
        regprog,
        startp: [std::ptr::null(); 10],
        endp: [std::ptr::null(); 10],
        rm_matchcol: 0,
        rm_ic: ignore_case,
    };
    vim_regexec(&mut rmp, line, col)
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
    let text_bytes = unsafe { CStr::from_ptr(text).to_bytes() };
    let sub_bytes = unsafe { CStr::from_ptr(sub).to_bytes() };
    let replaced = prog
        .regex
        .replace(text_bytes, NoExpand(sub_bytes))
        .into_owned();
    CString::new(replaced).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn vim_regcomp_had_eol() -> c_int {
    0
}
