use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};

// On Windows a path separator can also be ':'; match the C implementation of
// `vim_ispathsep` which treats "c:/" and "c:\\" style paths equally.
#[cfg(target_os = "windows")]
const PATH_SEPARATORS: &[char] = &['\\', '/', ':'];
#[cfg(not(target_os = "windows"))]
const PATH_SEPARATORS: &[char] = &['/'];

pub fn is_path_separator(ch: char) -> bool {
    PATH_SEPARATORS.contains(&ch)
}

pub fn join_paths(a: &str, b: &str) -> String {
    let mut path = PathBuf::from(a);
    path.push(b);
    path.to_string_lossy().into_owned()
}

#[no_mangle]
pub extern "C" fn rs_is_path_sep(ch: c_int) -> c_int {
    let ch = std::char::from_u32(ch as u32).unwrap_or('\0');
    is_path_separator(ch) as c_int
}

#[no_mangle]
pub extern "C" fn rs_path_join(a: *const c_char, b: *const c_char) -> *mut c_char {
    if a.is_null() || b.is_null() {
        return std::ptr::null_mut();
    }
    let a = unsafe { CStr::from_ptr(a) }.to_string_lossy().into_owned();
    let b = unsafe { CStr::from_ptr(b) }.to_string_lossy().into_owned();
    let joined = Path::new(&a).join(&b);
    CString::new(joined.to_string_lossy().into_owned()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn rs_path_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sep() {
        assert!(is_path_separator(std::path::MAIN_SEPARATOR));
    }

    #[test]
    fn join() {
        let joined = join_paths("a", "b");
        assert!(joined.ends_with("b"));
    }
}
