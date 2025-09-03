use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use similar::TextDiff;

#[repr(C)]
pub enum DiffMode {
    External = 0,
    Xdiff = 1,
}

fn read_file(path: &str) -> Result<Vec<u8>, ()> {
    std::fs::read(path).map_err(|_| ())
}

fn diff_files_internal(file1: &str, file2: &str) -> Result<CString, ()> {
    let a = read_file(file1)?;
    let b = read_file(file2)?;
    let text1 = String::from_utf8_lossy(&a);
    let text2 = String::from_utf8_lossy(&b);
    let diff = TextDiff::from_lines(&text1, &text2);
    let diff_str = diff.unified_diff().header(file1, file2).to_string();
    CString::new(diff_str).map_err(|_| ())
}

#[no_mangle]
pub extern "C" fn rs_diff_files(
    f1: *const c_char,
    f2: *const c_char,
    _mode: DiffMode,
) -> *mut c_char {
    if f1.is_null() || f2.is_null() {
        return ptr::null_mut();
    }
    let file1 = unsafe { CStr::from_ptr(f1) }.to_string_lossy().into_owned();
    let file2 = unsafe { CStr::from_ptr(f2) }.to_string_lossy().into_owned();
    match diff_files_internal(&file1, &file2) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rs_diff_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_diff_update_screen() {
    // Placeholder for integrating with screen updating.  Currently this just logs.
    eprintln!("rs_diff_update_screen called");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::ffi::{CStr, CString};
    use std::fs::write;
    #[test]
    fn library_diff() {
        let dir = temp_dir();
        let f1 = dir.join("a.txt");
        let f2 = dir.join("b.txt");
        write(&f1, "a\nb\n").unwrap();
        write(&f2, "a\nc\n").unwrap();
        let c1 = CString::new(f1.to_str().unwrap()).unwrap();
        let c2 = CString::new(f2.to_str().unwrap()).unwrap();
        // Mode is ignored internally but kept for API compatibility
        let ptr = rs_diff_files(c1.as_ptr(), c2.as_ptr(), DiffMode::External);
        assert!(!ptr.is_null());
        let diff = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
        rs_diff_free(ptr);
        assert!(diff.contains("-b"));
        assert!(diff.contains("+c"));
    }
}
