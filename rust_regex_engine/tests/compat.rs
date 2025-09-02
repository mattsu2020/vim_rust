use rust_regex_engine::{
    vim_regcomp, vim_regexec, vim_regexec_nl, vim_regfree, vim_regsub, RegMatch,
};
use std::ffi::{CStr, CString};

#[test]
fn basic_match_and_exec_nl() {
    let pat = CString::new("foo").unwrap();
    let text = CString::new("foo bar").unwrap();
    unsafe {
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
}

#[test]
fn substitution_and_flags() {
    let pat = CString::new("foo").unwrap();
    let text = CString::new("Foo bar").unwrap();
    unsafe {
        // enable case-insensitive match using flag bit 1
        let prog = vim_regcomp(pat.as_ptr(), 1);
        assert!(!prog.is_null());
        let sub = CString::new("baz").unwrap();
        let replaced = vim_regsub(prog, text.as_ptr(), sub.as_ptr());
        let c_str = CStr::from_ptr(replaced);
        assert_eq!(c_str.to_str().unwrap(), "baz bar");
        vim_regfree(prog);
    }
}

#[test]
fn capture_offsets() {
    let pat = CString::new("(ab)c").unwrap();
    let text = CString::new("zabc").unwrap();
    unsafe {
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
        // start and end pointers for whole match and group should be set
        assert!(!rm.startp[0].is_null());
        assert!(!rm.endp[1].is_null());
        vim_regfree(prog);
    }
}
