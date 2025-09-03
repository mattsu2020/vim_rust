use std::ffi::CStr;
use std::os::raw::c_char;

/// Check whether a language code starts with two ASCII alphabetic characters.
/// Mirrors `is_valid_mess_lang` from `locale.c`.
#[no_mangle]
pub extern "C" fn is_valid_mess_lang(lang: *const c_char) -> bool {
    if lang.is_null() {
        return false;
    }
    let bytes = unsafe { CStr::from_ptr(lang).to_bytes() };
    if bytes.len() < 2 {
        return false;
    }
    bytes[0].is_ascii_alphabetic() && bytes[1].is_ascii_alphabetic()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn detects_language() {
        let en = CString::new("en").unwrap();
        assert!(is_valid_mess_lang(en.as_ptr()));

        let c = CString::new("C").unwrap();
        assert!(!is_valid_mess_lang(c.as_ptr()));

        assert!(!is_valid_mess_lang(std::ptr::null()));
    }
}
