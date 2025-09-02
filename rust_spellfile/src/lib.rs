use std::ffi::{CStr};
use std::os::raw::{c_char, c_int};
use std::fs::{OpenOptions};
use std::io::{Write};
use std::path::Path;

const SPELL_ADD_GOOD: c_int = 0;
const SPELL_ADD_BAD: c_int = 1;
const SPELL_ADD_RARE: c_int = 2;

fn read_lines(path: &Path) -> Vec<String> {
    match std::fs::read_to_string(path) {
        Ok(content) => content.lines().map(|s| s.to_string()).collect(),
        Err(_) => Vec::new(),
    }
}

fn write_lines(path: &Path, lines: &[String]) -> std::io::Result<()> {
    let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(path)?;
    for (i, l) in lines.iter().enumerate() {
        if i + 1 == lines.len() {
            write!(file, "{}", l)?;
        } else {
            writeln!(file, "{}", l)?;
        }
    }
    Ok(())
}

#[no_mangle]
pub extern "C" fn rs_spellfile_add_word(
    fname: *const c_char,
    word: *const c_char,
    len: c_int,
    what: c_int,
    undo: c_int,
) -> c_int {
    if fname.is_null() || word.is_null() || len < 0 {
        return 0;
    }
    let fname_c = unsafe { CStr::from_ptr(fname) };
    let word_bytes = unsafe { std::slice::from_raw_parts(word as *const u8, len as usize) };
    let word_str = match std::str::from_utf8(word_bytes) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let fname_str = match fname_c.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let path = Path::new(fname_str);
    let mut lines = read_lines(path);
    if undo != 0 || what == SPELL_ADD_BAD {
        for l in lines.iter_mut() {
            if l.starts_with(word_str) && !l.starts_with('#') {
                *l = format!("#{}", l);
            }
        }
        if let Err(_) = write_lines(path, &lines) {
            return 0;
        }
    }
    if undo == 0 {
        let mut file = match OpenOptions::new().create(true).append(true).open(path) {
            Ok(f) => f,
            Err(_) => return 0,
        };
        let suffix = if what == SPELL_ADD_BAD {
            "/!"
        } else if what == SPELL_ADD_RARE {
            "/?"
        } else {
            ""
        };
        if writeln!(file, "{}{}", word_str, suffix).is_err() {
            return 0;
        }
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use tempfile::tempdir;

    #[test]
    fn add_and_undo() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.add");
        let fname = CString::new(path.to_str().unwrap()).unwrap();
        let word = CString::new("hello").unwrap();
        assert_eq!(rs_spellfile_add_word(fname.as_ptr(), word.as_ptr(), 5, SPELL_ADD_GOOD, 0), 1);
        assert_eq!(rs_spellfile_add_word(fname.as_ptr(), word.as_ptr(), 5, SPELL_ADD_GOOD, 1), 1);
        let contents = std::fs::read_to_string(path).unwrap();
        assert!(contents.contains("#hello"));
    }
}
