use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::{c_char, c_int};
use std::path::Path;

#[derive(Default)]
pub struct VimInfo {
    pub lines: Vec<String>,
}

pub fn read(path: &Path) -> std::io::Result<VimInfo> {
    let content = fs::read_to_string(path)?;
    Ok(VimInfo {
        lines: content.lines().map(|s| s.to_string()).collect(),
    })
}

pub fn write(path: &Path, info: &VimInfo) -> std::io::Result<()> {
    fs::write(path, info.lines.join("\n"))
}

#[no_mangle]
pub extern "C" fn rs_viminfo_read(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    match read(Path::new(&path)) {
        Ok(info) => CString::new(info.lines.join("\n")).unwrap().into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rs_viminfo_write(path: *const c_char, data: *const c_char) -> c_int {
    if path.is_null() || data.is_null() {
        return -1;
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    let data = unsafe { CStr::from_ptr(data) }.to_string_lossy();
    let info = VimInfo {
        lines: data.lines().map(|s| s.to_string()).collect(),
    };
    match write(Path::new(&path), &info) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn rs_viminfo_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn roundtrip() {
        let dir = env::temp_dir();
        let file = dir.join("viminfo_test.txt");
        let info = VimInfo { lines: vec!["alpha".into(), "beta".into()] };
        write(&file, &info).unwrap();
        let read_back = read(&file).unwrap();
        assert_eq!(read_back.lines, info.lines);
        fs::remove_file(file).unwrap();
    }
}
