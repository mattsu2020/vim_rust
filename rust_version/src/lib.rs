use std::ffi::{CStr};
use std::fs;
use std::os::raw::{c_char, c_int};
use std::path::Path;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub fn read(path: &Path) -> std::io::Result<Version> {
    let text = fs::read_to_string(path)?;
    let parts: Vec<_> = text.trim().split('.').collect();
    let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    Ok(Version { major, minor, patch })
}

pub fn write(path: &Path, v: Version) -> std::io::Result<()> {
    fs::write(path, v.to_string())
}

#[no_mangle]
pub extern "C" fn rs_version_read(path: *const c_char, major: *mut u32, minor: *mut u32, patch: *mut u32) -> c_int {
    if path.is_null() {
        return -1;
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    match read(Path::new(&path)) {
        Ok(v) => {
            unsafe {
                if !major.is_null() { *major = v.major; }
                if !minor.is_null() { *minor = v.minor; }
                if !patch.is_null() { *patch = v.patch; }
            }
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn rs_version_write(path: *const c_char, major: u32, minor: u32, patch: u32) -> c_int {
    if path.is_null() {
        return -1;
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    let v = Version { major, minor, patch };
    match write(Path::new(&path), v) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn roundtrip() {
        let dir = env::temp_dir();
        let file = dir.join("version_test.txt");
        let v = Version { major: 1, minor: 2, patch: 3 };
        write(&file, v).unwrap();
        let r = read(&file).unwrap();
        assert_eq!(r, v);
        fs::remove_file(file).unwrap();
    }
}
