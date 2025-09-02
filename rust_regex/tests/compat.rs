use std::ffi::CString;
use rust_regex::{RegMatch, vim_regcomp, vim_regexec, vim_regexec_nl, vim_regfree};

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
