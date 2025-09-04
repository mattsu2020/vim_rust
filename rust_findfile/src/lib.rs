use std::collections::HashSet;
use std::ffi::{CStr, CString, OsStr};
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

#[cfg(target_os = "windows")]
const PATH_LIST_SEPARATOR: char = ';';
#[cfg(not(target_os = "windows"))]
const PATH_LIST_SEPARATOR: char = ':';

pub fn find_file(file: &str, search: &str) -> Option<PathBuf> {
    for dir in search.split(PATH_LIST_SEPARATOR) {
        if dir.is_empty() { continue; }
        let candidate = PathBuf::from(dir).join(file);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[no_mangle]
pub extern "C" fn rs_findfile(file: *const c_char, search: *const c_char) -> *mut c_char {
    if file.is_null() || search.is_null() {
        return std::ptr::null_mut();
    }
    let file = unsafe { CStr::from_ptr(file) }.to_string_lossy().into_owned();
    let search = unsafe { CStr::from_ptr(search) }.to_string_lossy().into_owned();
    match find_file(&file, &search) {
        Some(path) => CString::new(path.to_string_lossy().into_owned()).unwrap().into_raw(),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rs_findfile_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}

/// Check if `fname` was already visited.  If not, add it to `visited`.
/// Returns 1 when the file was seen before, 0 otherwise.
#[no_mangle]
pub extern "C" fn ff_check_visited(
    visited: *mut *mut c_void,
    fname: *const c_char,
    fnamelen: usize,
    _wc_path: *const c_char,
    _wc_pathlen: usize,
) -> c_int {
    if visited.is_null() || fname.is_null() {
        return 0;
    }

    // Convert the path bytes to a PathBuf.
    let bytes = unsafe { std::slice::from_raw_parts(fname as *const u8, fnamelen) };
    let path = {
        #[cfg(unix)]
        { PathBuf::from(OsStr::from_bytes(bytes)) }
        #[cfg(not(unix))]
        { PathBuf::from(String::from_utf8_lossy(bytes).to_string()) }
    };

    unsafe {
        let set_ptr = *visited as *mut HashSet<PathBuf>;
        if set_ptr.is_null() {
            let mut set = match Box::try_new(HashSet::<PathBuf>::new()) {
                Ok(b) => b,
                Err(_) => {
                    eprintln!("warning: findfile: out of memory");
                    return 0;
                }
            };
            if set.try_reserve(1).is_err() {
                eprintln!("warning: findfile: out of memory");
                return 0;
            }
            set.insert(path);
            *visited = Box::into_raw(set) as *mut c_void;
            0
        } else {
            let set = &mut *set_ptr;
            if set.contains(&path) {
                1
            } else {
                if set.try_reserve(1).is_err() {
                    eprintln!("warning: findfile: out of memory");
                    0
                } else {
                    set.insert(path);
                    0
                }
            }
        }
    }
}

/// Free a visited list allocated in Rust.
#[no_mangle]
pub extern "C" fn ff_free_visited_list(list: *mut c_void) {
    if !list.is_null() {
        unsafe { drop(Box::from_raw(list as *mut HashSet<PathBuf>)); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found() {
        assert!(find_file("nonexistent", "").is_none());
    }
}
