use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::sync::{Mutex, OnceLock};

static WORDS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn rs_spell_add_word(word: *const c_char) {
    if word.is_null() {
        return;
    }
    let cstr = unsafe { CStr::from_ptr(word) };
    if let Ok(w) = cstr.to_str() {
        WORDS.get_or_init(|| Mutex::new(HashSet::new()))
            .lock()
            .unwrap()
            .insert(w.to_string());
    }
}

#[no_mangle]
pub extern "C" fn rs_spell_check(word: *const c_char) -> c_int {
    if word.is_null() {
        return 0;
    }
    let cstr = unsafe { CStr::from_ptr(word) };
    if let Ok(w) = cstr.to_str() {
        if let Some(set) = WORDS.get() {
            if set.lock().unwrap().contains(w) {
                return 1;
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn rs_spell_clear() {
    if let Some(set) = WORDS.get() {
        set.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn add_and_check() {
        let hello = CString::new("hello").unwrap();
        let world = CString::new("world").unwrap();
        rs_spell_clear();
        rs_spell_add_word(hello.as_ptr());
        assert_eq!(rs_spell_check(hello.as_ptr()), 1);
        assert_eq!(rs_spell_check(world.as_ptr()), 0);
        rs_spell_add_word(world.as_ptr());
        assert_eq!(rs_spell_check(world.as_ptr()), 1);
        rs_spell_clear();
        assert_eq!(rs_spell_check(hello.as_ptr()), 0);
    }
}
