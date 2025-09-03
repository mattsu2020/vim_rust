use std::ffi::CString;
use rust_regex_engine::{vim_regcomp, vim_regexec, vim_regfree, RegMatch};

unsafe fn run(pattern: &str, text: &str, flags: i32) -> Option<String> {
    let pat = CString::new(pattern).unwrap();
    let prog = vim_regcomp(pat.as_ptr(), flags);
    if prog.is_null() { return None; }
    let line = CString::new(text).unwrap();
    let mut rm = RegMatch { regprog: prog, startp: [std::ptr::null();10], endp: [std::ptr::null();10], rm_matchcol:0, rm_ic:0 };
    let ok = vim_regexec(&mut rm, line.as_ptr(), 0);
    let result = if ok == 1 {
        let start = rm.startp[0];
        let end = rm.endp[0];
        let len = end.offset_from(start) as usize;
        let slice = std::slice::from_raw_parts(start as *const u8, len);
        Some(String::from_utf8(slice.to_vec()).unwrap())
    } else { None };
    vim_regfree(prog);
    result
}

#[test]
fn backtrack_posix_digit() {
    unsafe {
        let m = run("[[:digit:]]+", "abc123def", 0).unwrap();
        assert_eq!(m, "123");
    }
}

#[test]
fn nfa_posix_digit() {
    unsafe {
        let m = run("[[:digit:]]+", "abc123def", 2).unwrap();
        assert_eq!(m, "123");
    }
}

#[test]
fn backtrack_plus_question() {
    unsafe {
        let m = run("colou?r", "colour", 0).unwrap();
        assert_eq!(m, "colour");
        let m2 = run("colou?r", "color", 0).unwrap();
        assert_eq!(m2, "color");
    }
}

#[test]
fn nfa_class_range() {
    unsafe {
        let m = run("^[a-c]+$", "abcc", 2).unwrap();
        assert_eq!(m, "abcc");
    }
}
