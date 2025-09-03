use once_cell::sync::Lazy;
use std::ffi::{CStr, CString};
use std::io::{BufRead, BufReader};
use std::os::raw::{c_char, c_int};
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Tag stack handling
// ---------------------------------------------------------------------------

static TAG_STACK: Lazy<Mutex<Vec<CString>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Push a tag name onto the stack. The string is copied and owned by the stack.
#[no_mangle]
pub extern "C" fn rust_tagstack_push(tag: *const c_char) {
    if tag.is_null() {
        return;
    }
    let s = unsafe { CStr::from_ptr(tag) }.to_owned();
    TAG_STACK.lock().unwrap().push(s);
}

/// Pop a tag name from the stack. The returned pointer should be freed by the
/// caller using `free()`.
#[no_mangle]
pub extern "C" fn rust_tagstack_pop() -> *mut c_char {
    match TAG_STACK.lock().unwrap().pop() {
        Some(s) => {
            let len = s.as_bytes_with_nul().len();
            unsafe {
                let mem = libc::malloc(len) as *mut c_char;
                if mem.is_null() {
                    return std::ptr::null_mut();
                }
                std::ptr::copy_nonoverlapping(s.as_ptr(), mem, len);
                mem
            }
        }
        None => std::ptr::null_mut(),
    }
}

/// Return the current size of the tag stack.
#[no_mangle]
pub extern "C" fn rust_tagstack_len() -> c_int {
    TAG_STACK.lock().unwrap().len() as c_int
}

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

#[cfg(test)]
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

    #[test]
    fn parses_ctags_style_lines() {
        let dir = tempfile::tempdir().unwrap();
        let tags_path = dir.path().join("tags");
        {
            let mut f = std::fs::File::create(&tags_path).unwrap();
            writeln!(f, "main\tmain.c\t/^int main()$/;\"\tf").unwrap();
            writeln!(f, "helper\thelper.c\t/^void helper()$/;\"\tf").unwrap();
        }
        let matches = search_tag(&tags_path, "main");
        assert_eq!(matches.len(), 1);
        assert!(matches[0].contains("main.c"));
    }

    #[test]
    fn tagstack_push_and_pop() {
        let s = CString::new("foo").unwrap();
        rust_tagstack_push(s.as_ptr());
        assert_eq!(rust_tagstack_len(), 1);
        let ptr = rust_tagstack_pop();
        assert!(!ptr.is_null());
        unsafe {
            assert_eq!(CStr::from_ptr(ptr).to_str().unwrap(), "foo");
            libc::free(ptr as *mut libc::c_void);
        }
        assert_eq!(rust_tagstack_len(), 0);
    }
}
