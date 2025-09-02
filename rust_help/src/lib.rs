use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::io::{BufRead, BufReader};

use rust_window::{rs_win_new, rs_win_free};

#[no_mangle]
pub extern "C" fn rs_find_help_tags(
    pat: *const c_char,
    num_matches: *mut c_int,
    matchesp: *mut *mut *mut c_char,
    _keep_lang: c_int,
) -> c_int {
    let Ok(pat) = (unsafe { CStr::from_ptr(pat).to_str() }) else {
        return 0;
    };

    let path = std::path::Path::new("doc/tags");
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => {
            unsafe {
                if !num_matches.is_null() {
                    *num_matches = 0;
                }
                if !matchesp.is_null() {
                    *matchesp = std::ptr::null_mut();
                }
            }
            return 0;
        }
    };

    let mut results: Vec<*mut c_char> = Vec::new();
    for line in BufReader::new(file).lines().flatten() {
        if line.starts_with(pat) && line.as_bytes().get(pat.len()) == Some(&b'\t') {
            if let Ok(cstr) = CString::new(line) {
                let len = cstr.as_bytes_with_nul().len();
                unsafe {
                    let mem = libc::malloc(len) as *mut c_char;
                    if mem.is_null() {
                        continue;
                    }
                    std::ptr::copy_nonoverlapping(cstr.as_ptr(), mem, len);
                    results.push(mem);
                }
            }
        }
    }

    unsafe {
        if !num_matches.is_null() {
            *num_matches = results.len() as c_int;
        }
        if !matchesp.is_null() {
            if results.is_empty() {
                *matchesp = std::ptr::null_mut();
            } else {
                let size = results.len() * std::mem::size_of::<*mut c_char>();
                let arr = libc::malloc(size) as *mut *mut c_char;
                if arr.is_null() {
                    *matchesp = std::ptr::null_mut();
                    if !num_matches.is_null() {
                        *num_matches = 0;
                    }
                    return 0;
                }
                for (i, ptr) in results.iter().enumerate() {
                    *arr.add(i) = *ptr;
                }
                *matchesp = arr;
            }
        }
    }

    1
}

#[no_mangle]
pub extern "C" fn rs_help_open_window(width: c_int, height: c_int) -> *mut c_void {
    let ptr = Box::into_raw(Box::new(0u8)) as *mut c_void;
    rs_win_new(ptr, width, height);
    ptr
}

#[no_mangle]
pub extern "C" fn rs_help_close_window(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    rs_win_free(ptr);
    unsafe {
        drop(Box::from_raw(ptr as *mut u8));
    }
}

fn search_help_tag(path: &std::path::Path, pat: &str) -> Vec<String> {
    let mut matches = Vec::new();
    if let Ok(file) = std::fs::File::open(path) {
        for line in BufReader::new(file).lines().flatten() {
            if line.starts_with(pat) && line.as_bytes().get(pat.len()) == Some(&b'\t') {
                matches.push(line);
            }
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use rust_window::rs_win_save;

    #[test]
    fn finds_help_tag() {
        let dir = tempfile::tempdir().unwrap();
        let docdir = dir.path().join("doc");
        std::fs::create_dir(&docdir).unwrap();
        let tags_path = docdir.join("tags");
        {
            let mut f = std::fs::File::create(&tags_path).unwrap();
            writeln!(f, "foo\tfile1\t1").unwrap();
            writeln!(f, "bar\tfile2\t1").unwrap();
        }
        let matches = search_help_tag(&tags_path, "foo");
        assert_eq!(matches.len(), 1);
        assert!(matches[0].starts_with("foo"));
    }

    #[test]
    fn open_and_close_window() {
        let ptr = rs_help_open_window(80, 24);
        let state = rs_win_save(ptr);
        assert_eq!(state.width, 80);
        assert_eq!(state.height, 24);
        rs_help_close_window(ptr);
    }
}

