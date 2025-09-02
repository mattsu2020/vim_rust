use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn rs_normalize_path(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let c_path = unsafe { CStr::from_ptr(path) };
    let result = match c_path.to_str() {
        Ok(p) => {
            let pb = std::path::Path::new(p);
            let norm = std::fs::canonicalize(pb).unwrap_or_else(|_| pb.to_path_buf());
            norm.to_string_lossy().into_owned()
        }
        Err(_) => return std::ptr::null_mut(),
    };
    CString::new(result).map(|s| s.into_raw()).unwrap_or_else(|_| std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn rs_find_in_path(name: *const c_char, paths: *const c_char) -> *mut c_char {
    if name.is_null() || paths.is_null() {
        return std::ptr::null_mut();
    }
    let c_name = unsafe { CStr::from_ptr(name) };
    let c_paths = unsafe { CStr::from_ptr(paths) };
    let file = match c_name.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let path_list = match c_paths.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    #[cfg(windows)]
    let sep = ';';
    #[cfg(not(windows))]
    let sep = ':';

    for dir in path_list.split(sep) {
        if dir.is_empty() {
            continue;
        }
        let mut pb = std::path::PathBuf::from(dir);
        pb.push(file);
        if pb.exists() {
            if let Some(s) = pb.to_str() {
                return CString::new(s).map(|c| c.into_raw()).unwrap_or(std::ptr::null_mut());
            }
        }
    }
    std::ptr::null_mut()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::fs::{self, File};
    use std::io::Write;
    use std::env;

    #[test]
    fn normalize_current_dir() {
        let cwd = env::current_dir().unwrap();
        let s = CString::new(".").unwrap();
        let ptr = rs_normalize_path(s.as_ptr());
        assert!(!ptr.is_null());
        let norm = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_string();
        unsafe { CString::from_raw(ptr) };
        assert_eq!(norm, cwd.to_string_lossy());
    }

    #[test]
    fn find_in_paths() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("foo.txt");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "hello").unwrap();

        let name = CString::new("foo.txt").unwrap();
        let paths = CString::new(dir.path().to_str().unwrap()).unwrap();
        let res_ptr = rs_find_in_path(name.as_ptr(), paths.as_ptr());
        assert!(!res_ptr.is_null());
        let res = unsafe { CStr::from_ptr(res_ptr) }.to_str().unwrap().to_string();
        unsafe { CString::from_raw(res_ptr) };
        assert_eq!(res, file_path.to_string_lossy());
    }
}
