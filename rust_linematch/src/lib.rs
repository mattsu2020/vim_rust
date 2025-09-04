use std::cmp::min;
use std::os::raw::{c_char, c_int};
use std::slice;

#[repr(C)]
pub struct MMFile {
    pub ptr: *const c_char,
    pub size: usize,
}

fn line_len(m: &MMFile) -> usize {
    if m.ptr.is_null() {
        return 0;
    }
    let bytes = unsafe { slice::from_raw_parts(m.ptr as *const u8, m.size) };
    match bytes.iter().position(|&b| b == b'\n') {
        Some(idx) => idx,
        None => bytes.len(),
    }
}

fn lcs(a: &[u8], b: &[u8]) -> usize {
    if a.is_empty() || b.is_empty() {
        return 0;
    }
    let n = b.len();
    let mut dp = vec![vec![0; n + 1]; 2];
    for (i, &ai) in a.iter().enumerate() {
        let cur = i % 2;
        let prev = 1 - cur;
        for (j, &bj) in b.iter().enumerate() {
            if ai == bj {
                dp[cur][j + 1] = dp[prev][j] + 1;
            } else {
                dp[cur][j + 1] = dp[cur][j].max(dp[prev][j + 1]);
            }
        }
    }
    dp[(a.len() - 1) % 2][n]
}

#[no_mangle]
pub extern "C" fn matching_chars(m1: *const MMFile, m2: *const MMFile) -> c_int {
    if m1.is_null() || m2.is_null() {
        return 0;
    }
    let m1 = unsafe { &*m1 };
    let m2 = unsafe { &*m2 };
    let s1len = min(799, line_len(m1));
    let s2len = min(799, line_len(m2));
    let s1 = unsafe { slice::from_raw_parts(m1.ptr as *const u8, s1len) };
    let s2 = unsafe { slice::from_raw_parts(m2.ptr as *const u8, s2len) };
    lcs(s1, s2) as c_int
}

fn strip_ws(m: &MMFile) -> Vec<u8> {
    let len = min(799, line_len(m));
    let bytes = unsafe { slice::from_raw_parts(m.ptr as *const u8, len) };
    bytes
        .iter()
        .cloned()
        .filter(|&b| b != b' ' && b != b'\t')
        .collect()
}

#[no_mangle]
pub extern "C" fn matching_chars_ignore_whitespace(m1: *const MMFile, m2: *const MMFile) -> c_int {
    if m1.is_null() || m2.is_null() {
        return 0;
    }
    let s1 = strip_ws(unsafe { &*m1 });
    let s2 = strip_ws(unsafe { &*m2 });
    lcs(&s1, &s2) as c_int
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_matching_chars_basic() {
        let s1 = CString::new("aabc").unwrap();
        let s2 = CString::new("acba").unwrap();
        let m1 = MMFile { ptr: s1.as_ptr(), size: s1.as_bytes().len() };
        let m2 = MMFile { ptr: s2.as_ptr(), size: s2.as_bytes().len() };
        assert_eq!(matching_chars(&m1, &m2), 2);
    }

    #[test]
    fn test_ignore_whitespace() {
        let s1 = CString::new("a b c").unwrap();
        let s2 = CString::new("abc").unwrap();
        let m1 = MMFile { ptr: s1.as_ptr(), size: s1.as_bytes().len() };
        let m2 = MMFile { ptr: s2.as_ptr(), size: s2.as_bytes().len() };
        assert_eq!(matching_chars_ignore_whitespace(&m1, &m2), 3);
    }
}
