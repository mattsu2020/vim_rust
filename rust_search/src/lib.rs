use regex::Regex;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct SearchStat {
    pub cur: c_int,
    pub cnt: c_int,
    pub exact_match: c_int,
    pub incomplete: c_int,
    pub last_maxcount: c_int,
}

#[no_mangle]
pub extern "C" fn rust_search_update_stat(
    pat: *const c_char,
    text: *const c_char,
    stat: *mut SearchStat,
) -> c_int {
    if pat.is_null() || text.is_null() || stat.is_null() {
        return 0;
    }

    let pat_c = unsafe { CStr::from_ptr(pat) };
    let text_c = unsafe { CStr::from_ptr(text) };

    let stat = unsafe { &mut *stat };
    stat.cur = 0;
    stat.cnt = 0;
    stat.exact_match = 0;
    stat.incomplete = 0;
    stat.last_maxcount = 0;

    let pat_str = match pat_c.to_str() {
        Ok(s) => s,
        Err(_) => {
            stat.incomplete = 1;
            return 0;
        }
    };
    let text_str = match text_c.to_str() {
        Ok(s) => s,
        Err(_) => {
            stat.incomplete = 1;
            return 0;
        }
    };

    let re = match Regex::new(pat_str) {
        Ok(r) => r,
        Err(_) => {
            stat.incomplete = 1;
            return 0;
        }
    };

    for (i, m) in re.find_iter(text_str).enumerate() {
        stat.cnt += 1;
        stat.last_maxcount = stat.cnt;
        if m.start() == 0 && m.end() == text_str.len() {
            stat.exact_match = 1;
        }
        if i == 0 {
            stat.cur = 1;
        }
    }

    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn make_stat() -> SearchStat {
        SearchStat { cur: 0, cnt: 0, exact_match: 0, incomplete: 0, last_maxcount: 0 }
    }

    #[test]
    fn counts_multiple_matches() {
        let pat = CString::new("foo").unwrap();
        let text = CString::new("foo foo").unwrap();
        let mut stat = make_stat();
        let res = rust_search_update_stat(pat.as_ptr(), text.as_ptr(), &mut stat);
        assert_eq!(res, 1);
        assert_eq!(stat.cnt, 2);
        assert_eq!(stat.cur, 1);
        assert_eq!(stat.exact_match, 0);
        assert_eq!(stat.incomplete, 0);
        assert_eq!(stat.last_maxcount, 2);
    }

    #[test]
    fn handles_no_match() {
        let pat = CString::new("foo").unwrap();
        let text = CString::new("bar").unwrap();
        let mut stat = make_stat();
        let res = rust_search_update_stat(pat.as_ptr(), text.as_ptr(), &mut stat);
        assert_eq!(res, 1);
        assert_eq!(stat.cnt, 0);
        assert_eq!(stat.cur, 0);
    }

    #[test]
    fn invalid_pattern_sets_incomplete() {
        let pat = CString::new("(").unwrap();
        let text = CString::new("foo").unwrap();
        let mut stat = make_stat();
        let res = rust_search_update_stat(pat.as_ptr(), text.as_ptr(), &mut stat);
        assert_eq!(res, 0);
        assert_eq!(stat.incomplete, 1);
    }

    #[test]
    fn non_overlapping_matches() {
        let pat = CString::new("aa").unwrap();
        let text = CString::new("aaa").unwrap();
        let mut stat = make_stat();
        rust_search_update_stat(pat.as_ptr(), text.as_ptr(), &mut stat);
        assert_eq!(stat.cnt, 1);
    }
}
