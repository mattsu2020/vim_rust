use std::ffi::CStr;
use std::os::raw::c_char;

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[no_mangle]
pub unsafe extern "C" fn cin_is_cinword(
    line: *const c_char,
    cinwords: *const c_char,
) -> bool {
    if line.is_null() || cinwords.is_null() {
        return false;
    }
    let line = match CStr::from_ptr(line).to_str() {
        Ok(s) => s.trim_start(),
        Err(_) => return false,
    };
    let cinwords = match CStr::from_ptr(cinwords).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    for word in cinwords.split(',').map(|w| w.trim()) {
        if line.starts_with(word) {
            let next = line[word.len()..].chars().next();
            if next.map_or(true, |c| !is_word_char(c)) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn detects_cinword() {
        let line = CString::new("if (x)").unwrap();
        let words = CString::new("if,while").unwrap();
        assert!(unsafe { cin_is_cinword(line.as_ptr(), words.as_ptr()) });
    }

    #[test]
    fn non_cinword() {
        let line = CString::new("foo").unwrap();
        let words = CString::new("if,while").unwrap();
        assert!(!unsafe { cin_is_cinword(line.as_ptr(), words.as_ptr()) });
    }

    #[test]
    fn word_boundary() {
        let line = CString::new("ifdef").unwrap();
        let words = CString::new("if,while").unwrap();
        assert!(!unsafe { cin_is_cinword(line.as_ptr(), words.as_ptr()) });
    }
}
