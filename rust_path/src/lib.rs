use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

pub fn normalize_path(path: &str) -> Option<String> {
    let pb = Path::new(path);
    let norm = std::fs::canonicalize(pb).unwrap_or_else(|_| pb.to_path_buf());
    Some(norm.to_string_lossy().into_owned())
}

pub fn find_in_path(name: &str, paths: &str) -> Option<String> {
    for dir in env::split_paths(paths) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let candidate: PathBuf = dir.join(name);
        if candidate.exists() {
            if let Some(s) = candidate.to_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}

#[no_mangle]
pub extern "C" fn rs_normalize_path(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let c_path = unsafe { CStr::from_ptr(path) };
    let result = match c_path.to_str() {
        Ok(p) => normalize_path(p),
        Err(_) => None,
    };
    match result {
        Some(r) => CString::new(r)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rs_find_in_path(name: *const c_char, paths: *const c_char) -> *mut c_char {
    if name.is_null() || paths.is_null() {
        return std::ptr::null_mut();
    }
    let c_name = unsafe { CStr::from_ptr(name) };
    let c_paths = unsafe { CStr::from_ptr(paths) };
    let res = match (c_name.to_str(), c_paths.to_str()) {
        (Ok(n), Ok(p)) => find_in_path(n, p),
        _ => None,
    };
    match res {
        Some(s) => CString::new(s)
            .map(|c| c.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        None => std::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::ffi::CString;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn normalize_current_dir() {
        let cwd = env::current_dir().unwrap();
        let s = CString::new(".").unwrap();
        let ptr = rs_normalize_path(s.as_ptr());
        assert!(!ptr.is_null());
        let norm = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_string();
        unsafe {
            let _ = CString::from_raw(ptr);
        };
        assert_eq!(norm, cwd.to_string_lossy());
    }

    #[test]
    fn find_in_paths() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("foo.txt");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "hello").unwrap();

        let name = CString::new("foo.txt").unwrap();
        let paths_os = env::join_paths([dir.path()]).unwrap();
        let paths = CString::new(paths_os.to_str().unwrap()).unwrap();
        let res_ptr = rs_find_in_path(name.as_ptr(), paths.as_ptr());
        assert!(!res_ptr.is_null());
        let res = unsafe { CStr::from_ptr(res_ptr) }
            .to_str()
            .unwrap()
            .to_string();
        unsafe {
            let _ = CString::from_raw(res_ptr);
        };
        assert_eq!(res, file_path.to_string_lossy());
    }

    #[test]
    fn normalize_relative_components() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("subdir");
        std::fs::create_dir(&nested).unwrap();
        let rel = nested.join("..");
        let c_path = CString::new(rel.to_str().unwrap()).unwrap();
        let ptr = rs_normalize_path(c_path.as_ptr());
        assert!(!ptr.is_null());
        let norm = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_string();
        unsafe { let _ = CString::from_raw(ptr); }
        assert_eq!(norm, dir.path().canonicalize().unwrap().to_string_lossy());
    }

    #[cfg(windows)]
    #[test]
    fn normalize_backslashes() {
        let tmp = CString::new("C:\\Windows\\.\\").unwrap();
        let ptr = rs_normalize_path(tmp.as_ptr());
        assert!(!ptr.is_null());
        let norm = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_string();
        unsafe {
            let _ = CString::from_raw(ptr);
        };
        assert!(norm.contains('\\'));
    }
}
