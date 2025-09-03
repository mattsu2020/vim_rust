use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Iterator over paths under a directory.
pub struct PathIter {
    inner: walkdir::IntoIter,
}

impl PathIter {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        PathIter { inner: WalkDir::new(root).into_iter() }
    }
}

impl Iterator for PathIter {
    type Item = PathBuf;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entry) = self.inner.next() {
            if let Ok(e) = entry {
                return Some(e.path().to_path_buf());
            }
        }
        None
    }
}

/// Find the first file named `target` under `start` directory.
#[no_mangle]
pub extern "C" fn rust_findfile(start: *const c_char, target: *const c_char) -> *mut c_char {
    if start.is_null() || target.is_null() {
        return std::ptr::null_mut();
    }
    let start_c = unsafe { CStr::from_ptr(start) };
    let target_c = unsafe { CStr::from_ptr(target) };
    let start_str = match start_c.to_str() { Ok(s) => s, Err(_) => return std::ptr::null_mut() };
    let target_str = match target_c.to_str() { Ok(s) => s, Err(_) => return std::ptr::null_mut() };

    for entry in WalkDir::new(start_str).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == target_str {
            if let Some(s) = entry.path().to_str() {
                if let Ok(c) = CString::new(s) {
                    return c.into_raw();
                }
            }
            break;
        }
    }
    std::ptr::null_mut()
}

/// Simple fuzzy matcher using iterators.
pub fn fuzzy<'a, I>(pattern: &'a str, iter: I) -> impl Iterator<Item = String> + 'a
where
    I: IntoIterator<Item = String> + 'a,
{
    iter.into_iter().filter(move |cand| is_subsequence(pattern, cand))
}

fn is_subsequence(pattern: &str, text: &str) -> bool {
    let mut chars = pattern.chars();
    let mut current = match chars.next() { Some(c) => c, None => return true };
    for t in text.chars() {
        if t == current {
            if let Some(n) = chars.next() {
                current = n;
            } else {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn iterates_paths() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        fs::File::create(&file_path).unwrap();

        let mut iter = PathIter::new(dir.path());
        let mut found = false;
        while let Some(p) = iter.next() {
            if p == file_path { found = true; break; }
        }
        assert!(found);
    }

    #[test]
    fn fuzzy_matches() {
        let items = vec!["foo".to_string(), "bar".to_string(), "fbar".to_string()];
        let res: Vec<_> = fuzzy("fb", items).collect();
        assert_eq!(res, vec!["fbar".to_string()]);
    }
}
