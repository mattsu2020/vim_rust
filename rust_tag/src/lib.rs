use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::io::{BufRead, BufReader};

/// Search the file named "tags" in the current directory for entries whose
/// tag name matches `pat`. Each matching line is returned to C as an allocated
/// C string. The caller is responsible for freeing both the array and the
/// individual strings with `free()`.
#[no_mangle]
pub extern "C" fn rust_find_tags(
    pat: *const c_char,
    num_matches: *mut c_int,
    matchesp: *mut *mut *mut c_char,
    _flags: c_int,
    _mincount: c_int,
    _buf_ffname: *const c_char,
) -> c_int {
    // Safety: the pointers are expected to be valid C strings.
    let Ok(pat) = (unsafe { CStr::from_ptr(pat).to_str() }) else {
        return 0;
    };

    let file = match std::fs::File::open("tags") {
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

fn search_tag(path: &std::path::Path, pat: &str) -> Vec<String> {
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

    #[test]
    fn finds_simple_tag() {
        let dir = tempfile::tempdir().unwrap();
        let tags_path = dir.path().join("tags");
        {
            let mut f = std::fs::File::create(&tags_path).unwrap();
            writeln!(f, "foo\tfile1\t1").unwrap();
            writeln!(f, "bar\tfile2\t1").unwrap();
        }
        let matches = search_tag(&tags_path, "foo");
        assert_eq!(matches.len(), 1);
        assert!(matches[0].starts_with("foo"));
    }
}
