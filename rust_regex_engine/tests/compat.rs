use rust_regex_engine::{
    vim_regcomp, vim_regexec, vim_regexec_multi, vim_regexec_nl, vim_regfree, vim_regsub, Lpos,
    RegMMMatch, RegMatch,
};
use std::ffi::{CStr, CString};
use std::os::raw::c_void;

#[test]
fn basic_match_and_exec_nl() {
    let pat = CString::new("foo").unwrap();
    let text = CString::new("foo bar").unwrap();
    let prog = vim_regcomp(pat.as_ptr(), 0);
    assert!(!prog.is_null());
    let mut rm = RegMatch {
        regprog: prog,
        startp: [std::ptr::null(); 10],
        endp: [std::ptr::null(); 10],
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
    let mut rm = RegMatch {
        regprog: prog,
        startp: [std::ptr::null(); 10],
        endp: [std::ptr::null(); 10],
        rm_matchcol: 0,
        rm_ic: 0,
    };
    assert_eq!(vim_regexec(&mut rm, text.as_ptr(), 0), 1);
    assert!(!rm.startp[0].is_null());
    assert!(!rm.endp[0].is_null());
    assert!(!rm.startp[1].is_null());
    assert!(!rm.endp[1].is_null());
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
    let mut rmm = RegMMMatch {
        regprog: prog,
        startpos: [Lpos { lnum: 0, col: 0 }; 10],
        endpos: [Lpos { lnum: 0, col: 0 }; 10],
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
    assert_eq!(rmm.startpos[0].lnum, 2);
    assert_eq!(rmm.startpos[0].col, 0);
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
    let mut rm = RegMatch {
        regprog: prog,
        startp: [std::ptr::null(); 10],
        endp: [std::ptr::null(); 10],
        rm_matchcol: 0,
        rm_ic: 0,
    };
    assert_eq!(vim_regexec(&mut rm, text.as_ptr(), 0), 0);
    vim_regfree(prog);
}
