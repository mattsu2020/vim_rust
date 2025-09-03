use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::{PathBuf};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found() {
        assert!(find_file("nonexistent", "").is_none());
    }
}
