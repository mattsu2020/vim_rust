use libc::{c_int, c_long, size_t, c_uchar};
use regex::Regex;
use std::ffi::{CStr, CString};

extern "C" {
    fn ml_get_buf(buf: *mut buf_T, lnum: c_long, will_change: c_int) -> *const c_uchar;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: c_long) -> c_int;
    fn report_match(path: *const c_uchar);
}

#[repr(C)]
pub struct win_T { _private: [u8; 0] }
#[repr(C)]
pub struct buf_T { _private: [u8; 0] }
#[repr(C)]
pub struct searchit_arg_T { _private: [u8; 0] }
#[repr(C)]
pub struct oparg_T { _private: [u8; 0] }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct pos_T {
    pub lnum: c_long,
    pub col: c_int,
    pub coladd: c_int,
}

#[no_mangle]
pub extern "C" fn rust_searchit(
    _win: *mut win_T,
    buf: *mut buf_T,
    pos: *mut pos_T,
    end_pos: *mut pos_T,
    dir: c_int,
    pat: *const c_uchar,
    patlen: size_t,
    _count: c_long,
    _options: c_int,
    _pat_use: c_int,
    _extra_arg: *mut searchit_arg_T,
) -> c_int {
    if pos.is_null() || pat.is_null() || patlen == 0 {
        return 0;
    }

    let pat_slice = unsafe { std::slice::from_raw_parts(pat, patlen) };
    let pat_str = match std::str::from_utf8(pat_slice) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let re = match Regex::new(pat_str) {
        Ok(r) => r,
        Err(_) => return 0,
    };

    let lnum = unsafe { (*pos).lnum };
    let line_ptr = unsafe { ml_get_buf(buf, lnum, 0) };
    if line_ptr.is_null() {
        return 0;
    }
    let line_len = unsafe { ml_get_buf_len(buf, lnum) } as usize;
    let line_slice = unsafe { std::slice::from_raw_parts(line_ptr, line_len) };
    let line_str = match std::str::from_utf8(line_slice) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let mut start = unsafe { (*pos).col.max(0) } as usize;
    if start > line_str.len() {
        start = line_str.len();
    }
    let search_area = if dir >= 0 {
        let end_col = if !end_pos.is_null() && unsafe { (*end_pos).col } > 0 {
            unsafe { (*end_pos).col as usize }
        } else {
            line_str.len()
        };
        &line_str[start..end_col.min(line_str.len())]
    } else {
        &line_str[..start]
    };

    if dir >= 0 {
        if let Some(m) = re.find(search_area) {
            unsafe {
                (*pos).col = (start + m.start()) as c_int;
                if !end_pos.is_null() {
                    (*end_pos).lnum = lnum;
                    (*end_pos).col = (start + m.end()) as c_int;
                }
            }
            1
        } else {
            0
        }
    } else {
        let mut found = None;
        for m in re.find_iter(search_area) {
            found = Some(m);
        }
        if let Some(m) = found {
            unsafe {
                (*pos).col = m.start() as c_int;
                if !end_pos.is_null() {
                    (*end_pos).lnum = lnum;
                    (*end_pos).col = m.end() as c_int;
                }
            }
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_find_pattern_in_path(
    ptr: *mut c_uchar,
    _dir: c_int,
    _len: c_int,
    _whole: c_int,
    _skip_comments: c_int,
    _typ: c_int,
    _count: c_long,
    _action: c_int,
    _start_lnum: c_long,
    _end_lnum: c_long,
    _forceit: c_int,
    _silent: c_int,
) {
    if ptr.is_null() {
        return;
    }
    let pat = unsafe { CStr::from_ptr(ptr as *const i8) };
    let pat_str = match pat.to_str() {
        Ok(s) => s,
        Err(_) => return,
    };
    let re = match Regex::new(pat_str) {
        Ok(r) => r,
        Err(_) => return,
    };
    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(path_str) = p.to_str() {
                if re.is_match(path_str) {
                    if let Ok(c_path) = CString::new(path_str) {
                        unsafe { report_match(c_path.as_ptr() as *const c_uchar) };
                    }
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_do_search(
    win: *mut win_T,
    buf: *mut buf_T,
    cursor: *mut pos_T,
    _oap: *mut oparg_T,
    dirc: c_int,
    _search_delim: c_int,
    pat: *mut c_uchar,
    patlen: size_t,
    count: c_long,
    options: c_int,
    sia: *mut searchit_arg_T,
) -> c_int {
    if cursor.is_null() || pat.is_null() {
        return 0;
    }
    let mut pos = unsafe { *cursor };
    let mut end = pos_T { lnum: 0, col: 0, coladd: 0 };
    let dir = if dirc == b'?' as c_int { -1 } else { 1 };
    let res = rust_searchit(win, buf, &mut pos, &mut end, dir,
                            pat, patlen, count, options, 0, sia);
    if res > 0 {
        unsafe { *cursor = pos; }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::ffi::{CStr, CString};
    use std::ptr;

    thread_local! {
        static LINES: RefCell<Vec<&'static str>> = RefCell::new(vec!["hello world", "goodbye world"]);
        static PATH_RESULTS: RefCell<Vec<String>> = RefCell::new(Vec::new());
    }

    #[no_mangle]
    pub extern "C" fn ml_get_buf(_buf: *mut buf_T, lnum: c_long, _will_change: c_int) -> *const c_uchar {
        LINES.with(|l| l.borrow()[lnum as usize - 1].as_ptr())
    }

    #[no_mangle]
    pub extern "C" fn ml_get_buf_len(_buf: *mut buf_T, lnum: c_long) -> c_int {
        LINES.with(|l| l.borrow()[lnum as usize - 1].len() as c_int)
    }

    #[no_mangle]
    pub extern "C" fn report_match(path: *const c_uchar) {
        let s = unsafe { CStr::from_ptr(path as *const i8).to_string_lossy().into_owned() };
        PATH_RESULTS.with(|r| r.borrow_mut().push(s));
    }

    #[test]
    fn searchit_regex_forward() {
        let mut pos = pos_T { lnum: 1, col: 0, coladd: 0 };
        let mut end = pos_T { lnum: 0, col: 0, coladd: 0 };
        let pat = CString::new("world").unwrap();
        let r = unsafe {
            rust_searchit(
                ptr::null_mut(),
                ptr::null_mut(),
                &mut pos,
                &mut end,
                1,
                pat.as_ptr() as *const u8,
                5,
                1,
                0,
                0,
                ptr::null_mut(),
            )
        };
        assert_eq!(r, 1);
        assert_eq!(pos.col, 6);
        assert_eq!(end.col, 11);
    }

    #[test]
    fn searchit_compat_with_c_regex() {
        use libc::{regcomp, regexec, regfree, regex_t, regmatch_t, REG_EXTENDED};

        let line = LINES.with(|l| l.borrow()[0].to_string());
        let pattern = "world";

        // C regex
        let mut reg: regex_t = unsafe { std::mem::zeroed() };
        let cpat = CString::new(pattern).unwrap();
        assert_eq!(unsafe { regcomp(&mut reg, cpat.as_ptr(), REG_EXTENDED) }, 0);
        let cline = CString::new(line.clone()).unwrap();
        let mut pmatch: regmatch_t = unsafe { std::mem::zeroed() };
        let c_ret = unsafe { regexec(&mut reg, cline.as_ptr(), 1, &mut pmatch, 0) };
        unsafe { regfree(&mut reg) };
        assert_eq!(c_ret, 0);
        let c_start = pmatch.rm_so as usize;
        let c_end = pmatch.rm_eo as usize;

        let mut pos = pos_T { lnum: 1, col: 0, coladd: 0 };
        let mut end = pos_T { lnum: 0, col: 0, coladd: 0 };
        let r = unsafe {
            rust_searchit(
                ptr::null_mut(),
                ptr::null_mut(),
                &mut pos,
                &mut end,
                1,
                pattern.as_ptr(),
                pattern.len(),
                1,
                0,
                0,
                ptr::null_mut(),
            )
        };
        assert_eq!(r, 1);
        assert_eq!(pos.col as usize, c_start);
        assert_eq!(end.col as usize, c_end);
    }

    #[test]
    fn find_pattern_in_path_reports_matches() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("foo.txt"), b"").unwrap();
        std::fs::write(dir.path().join("bar.rs"), b"").unwrap();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        PATH_RESULTS.with(|r| r.borrow_mut().clear());
        let pat = CString::new("foo").unwrap();
        unsafe {
            rust_find_pattern_in_path(
                pat.as_ptr() as *mut u8,
                0,
                3,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            );
        }
        let results = PATH_RESULTS.with(|r| r.borrow().clone());
        assert!(results.iter().any(|s| s.ends_with("foo.txt")));
        std::env::set_current_dir(old).unwrap();
    }
}
