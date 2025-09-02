use std::ffi::CStr;
use std::fs;
use std::os::raw::{c_char, c_int};
use std::path::Path;

fn search_dir(dir: &Path, name: &str) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if search_dir(&path, name) {
                    return true;
                }
            } else if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
                if fname == name {
                    return true;
                }
            }
        }
    }
    false
}

/// Recursively search `dir` for `name`.
/// Returns 1 if found, 0 if not, and -1 on error.
#[no_mangle]
pub extern "C" fn rs_findfile(dir: *const c_char, name: *const c_char) -> c_int {
    if dir.is_null() || name.is_null() {
        return -1;
    }
    let c_dir = unsafe { CStr::from_ptr(dir) };
    let c_name = unsafe { CStr::from_ptr(name) };
    match (c_dir.to_str(), c_name.to_str()) {
        (Ok(d), Ok(n)) => {
            if search_dir(Path::new(d), n) {
                1
            } else {
                0
            }
        }
        _ => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::fs;

    #[test]
    fn find_existing_file() {
        let base = "tmp_findfile";
        let nested = format!("{}/subdir", base);
        fs::create_dir_all(&nested).unwrap();
        let target = format!("{}/target.txt", nested);
        fs::write(&target, b"hi").unwrap();

        let dir = CString::new(base).unwrap();
        let name = CString::new("target.txt").unwrap();
        assert_eq!(rs_findfile(dir.as_ptr(), name.as_ptr()), 1);

        let missing = CString::new("missing.txt").unwrap();
        assert_eq!(rs_findfile(dir.as_ptr(), missing.as_ptr()), 0);

        fs::remove_dir_all(base).unwrap();
    }
}
