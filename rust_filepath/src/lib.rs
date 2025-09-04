use std::env;
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};

use dialoguer::Select;

#[cfg(target_os = "windows")]
const PATH_SEPARATORS: &[char] = &['\\', '/'];
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

pub fn select_file_console(initdir: &Path) -> Option<String> {
    let mut entries = fs::read_dir(initdir).ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect::<Vec<String>>();
    entries.sort();
    if entries.is_empty() {
        return None;
    }

    if let Ok(choice) = env::var("TEST_SELECT_CHOICE") {
        if let Ok(idx) = choice.parse::<usize>() {
            if idx < entries.len() {
                return Some(initdir.join(&entries[idx]).to_string_lossy().into_owned());
            }
        }
        return None;
    }

    let selection = Select::new()
        .items(&entries)
        .default(0)
        .interact()
        .ok()?;
    Some(initdir.join(&entries[selection]).to_string_lossy().into_owned())
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

#[no_mangle]
pub extern "C" fn rs_select_file_console(initdir: *const c_char) -> *mut c_char {
    let dir = if initdir.is_null() {
        PathBuf::from(".")
    } else {
        let s = unsafe { CStr::from_ptr(initdir) }.to_string_lossy().into_owned();
        PathBuf::from(s)
    };
    match select_file_console(&dir) {
        Some(path) => CString::new(path).unwrap().into_raw(),
        None => std::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;
    use std::env;

    #[test]
    fn sep() {
        assert!(is_path_separator(std::path::MAIN_SEPARATOR));
    }

    #[test]
    fn join() {
        let joined = join_paths("a", "b");
        assert!(joined.ends_with("b"));
    }

    #[test]
    fn select_env() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("one.txt")).unwrap();
        File::create(dir.path().join("two.txt")).unwrap();
        env::set_var("TEST_SELECT_CHOICE", "1");
        let selected = select_file_console(dir.path()).unwrap();
        assert!(selected.ends_with("two.txt"));
        env::remove_var("TEST_SELECT_CHOICE");
    }
}
