use rust_regex_engine::{
    max_submatches, vim_regcomp, vim_regexec, vim_regexec_multi, vim_regexec_nl, vim_regfree,
    vim_regsub, Lpos, RegMMMatch, RegMatch,
};
use std::ffi::{CStr, CString};
use std::os::raw::c_void;

#[test]
fn basic_match_and_exec_nl() {
    let pat = CString::new("foo").unwrap();
    let text = CString::new("foo bar").unwrap();
    let prog = vim_regcomp(pat.as_ptr(), 0);
    assert!(!prog.is_null());
    let len = max_submatches();
    let mut startp = vec![std::ptr::null(); len];
    let mut endp = vec![std::ptr::null(); len];
    let mut rm = RegMatch {
        regprog: prog,
        startp: startp.as_mut_ptr(),
        endp: endp.as_mut_ptr(),
        len: len as i32,
        rm_matchcol: 0,
        rm_ic: 0,
    };
    assert_eq!(vim_regexec(&mut rm, text.as_ptr(), 0), 1);
    assert_eq!(vim_regexec_nl(&mut rm, text.as_ptr(), 0), 1);
    vim_regfree(prog);
}

#[test]
fn substitution_and_flags() {
    let pat = CString::new("foo").unwrap();
    let text = CString::new("Foo bar").unwrap();
    // enable case-insensitive match using flag bit 1
    let prog = vim_regcomp(pat.as_ptr(), 1);
    assert!(!prog.is_null());
    let sub = CString::new("baz").unwrap();
    let replaced = vim_regsub(prog, text.as_ptr(), sub.as_ptr());
    let c_str = unsafe { CStr::from_ptr(replaced) };
    assert_eq!(c_str.to_str().unwrap(), "baz bar");
    unsafe {
        let _ = CString::from_raw(replaced);
    }; // free result
    vim_regfree(prog);
}

#[test]
fn capture_offsets() {
    // Use a pattern with a capturing group and ensure offsets for the
    // whole match and the first group are filled in.
    let pat = CString::new("(a.)c").unwrap();
    let text = CString::new("zabc").unwrap();
    let prog = vim_regcomp(pat.as_ptr(), 0);
    assert!(!prog.is_null());
    let len = max_submatches();
    let mut startp = vec![std::ptr::null(); len];
    let mut endp = vec![std::ptr::null(); len];
    let mut rm = RegMatch {
        regprog: prog,
        startp: startp.as_mut_ptr(),
        endp: endp.as_mut_ptr(),
        len: len as i32,
        rm_matchcol: 0,
        rm_ic: 0,
    };
    assert_eq!(vim_regexec(&mut rm, text.as_ptr(), 0), 1);
    let start_slice = unsafe { std::slice::from_raw_parts(rm.startp, len) };
    let end_slice = unsafe { std::slice::from_raw_parts(rm.endp, len) };
    assert!(!start_slice[0].is_null());
    assert!(!end_slice[0].is_null());
    assert!(!start_slice[1].is_null());
    assert!(!end_slice[1].is_null());
    vim_regfree(prog);
}

#[test]
fn regexec_multi_single_line() {
    let pat = CString::new("bar").unwrap();
    let line1 = CString::new("foo").unwrap();
    let line2 = CString::new("bar baz").unwrap();
    let lines = [line1.as_ptr(), line2.as_ptr()];
    let prog = vim_regcomp(pat.as_ptr(), 0);
    assert!(!prog.is_null());
    let len = max_submatches();
    let mut startpos = vec![Lpos { lnum: 0, col: 0 }; len];
    let mut endpos = vec![Lpos { lnum: 0, col: 0 }; len];
    let mut rmm = RegMMMatch {
        regprog: prog,
        startpos: startpos.as_mut_ptr(),
        endpos: endpos.as_mut_ptr(),
        len: len as i32,
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    let matched = vim_regexec_multi(
        &mut rmm,
        std::ptr::null_mut(),
        lines.as_ptr() as *mut c_void,
        2,
        0,
        std::ptr::null_mut(),
    );
    assert_eq!(matched, 2);
    let start_slice = unsafe { std::slice::from_raw_parts(rmm.startpos, len) };
    assert_eq!(start_slice[0].lnum, 2);
    assert_eq!(start_slice[0].col, 0);
    vim_regfree(prog);
}

#[test]
fn regexec_multi_no_match() {
    let pat = CString::new("qux").unwrap();
    let line = CString::new("foo").unwrap();
    let lines = [line.as_ptr()];
    let prog = vim_regcomp(pat.as_ptr(), 0);
    assert!(!prog.is_null());
    let len = max_submatches();
    let mut startpos = vec![Lpos { lnum: 0, col: 0 }; len];
    let mut endpos = vec![Lpos { lnum: 0, col: 0 }; len];
    let mut rmm = RegMMMatch {
        regprog: prog,
        startpos: startpos.as_mut_ptr(),
        endpos: endpos.as_mut_ptr(),
        len: len as i32,
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    let matched = vim_regexec_multi(
        &mut rmm,
        std::ptr::null_mut(),
        lines.as_ptr() as *mut c_void,
        1,
        0,
        std::ptr::null_mut(),
    );
    assert_eq!(matched, 0);
    vim_regfree(prog);
}

#[test]
fn invalid_pattern_returns_null() {
    let pat = CString::new("[a-").unwrap();
    let prog = vim_regcomp(pat.as_ptr(), 0);
    assert!(prog.is_null());
}

#[test]
fn non_match_returns_zero() {
    let pat = CString::new("foo").unwrap();
    let text = CString::new("bar").unwrap();
    let prog = vim_regcomp(pat.as_ptr(), 0);
    assert!(!prog.is_null());
    let len = max_submatches();
    let mut startp = vec![std::ptr::null(); len];
    let mut endp = vec![std::ptr::null(); len];
    let mut rm = RegMatch {
        regprog: prog,
        startp: startp.as_mut_ptr(),
        endp: endp.as_mut_ptr(),
        len: len as i32,
        rm_matchcol: 0,
        rm_ic: 0,
    };
    assert_eq!(vim_regexec(&mut rm, text.as_ptr(), 0), 0);
    vim_regfree(prog);
}
