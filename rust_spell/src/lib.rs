use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uchar};
use std::sync::{Mutex, OnceLock};

use rust_spellfile::{load_dict, Trie};
use rust_spellsuggest::suggest;

const WF_ONECAP: c_int = 0x02;
const WF_ALLCAP: c_int = 0x04;
const WF_KEEPCAP: c_int = 0x80;

static TRIE: OnceLock<Mutex<Trie>> = OnceLock::new();

fn trie() -> &'static Mutex<Trie> {
    TRIE.get_or_init(|| Mutex::new(Trie::new()))
}

#[no_mangle]
pub extern "C" fn rs_spell_load_dict(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(path) };
    let Ok(p) = cstr.to_str() else { return false; };
    match load_dict(p) {
        Ok(t) => {
            *trie().lock().unwrap() = t;
            true
        }
        Err(_) => false,
    }
}

#[no_mangle]
pub extern "C" fn rs_spell_suggest(
    word: *const c_char,
    max: usize,
    len: *mut usize,
) -> *mut *mut c_char {
    if word.is_null() || len.is_null() {
        return std::ptr::null_mut();
    }
    let cstr = unsafe { CStr::from_ptr(word) };
    let Ok(w) = cstr.to_str() else { return std::ptr::null_mut(); };
    let suggestions = {
        let trie_guard = trie().lock().unwrap();
        suggest(&*trie_guard, w, max)
    };
    unsafe { *len = suggestions.len(); }
    let mut c_vec: Vec<*mut c_char> = suggestions
        .into_iter()
        .filter_map(|s| CString::new(s).ok().map(|cs| cs.into_raw()))
        .collect();
    let ptr = c_vec.as_mut_ptr();
    std::mem::forget(c_vec);
    ptr
}

#[no_mangle]
pub extern "C" fn rs_spell_free_suggestions(ptr: *mut *mut c_char, len: usize) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let v = Vec::from_raw_parts(ptr, len, len);
        for p in v {
            if !p.is_null() {
                drop(CString::from_raw(p));
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn captype(word: *const c_uchar, end: *const c_uchar) -> c_int {
    if word.is_null() {
        return 0;
    }
    unsafe {
        let len = if end.is_null() {
            let mut l = 0;
            while *word.add(l) != 0 {
                l += 1;
            }
            l
        } else {
            end.offset_from(word) as usize
        };
        let bytes = std::slice::from_raw_parts(word, len);
        let mut idx = 0;
        while idx < bytes.len() && !bytes[idx].is_ascii_alphabetic() {
            idx += 1;
        }
        if idx >= bytes.len() {
            return 0;
        }
        let first = bytes[idx];
        let firstcap = first.is_ascii_uppercase();
        let mut allcap = firstcap;
        idx += 1;
        let mut past_second = false;
        while idx < bytes.len() {
            let c = bytes[idx];
            idx += 1;
            if !c.is_ascii_alphabetic() {
                continue;
            }
            if !c.is_ascii_uppercase() {
                if past_second && allcap {
                    return WF_KEEPCAP;
                }
                allcap = false;
            } else if !allcap {
                return WF_KEEPCAP;
            }
            past_second = true;
        }
        if allcap {
            WF_ALLCAP
        } else if firstcap {
            WF_ONECAP
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::fs::File;
    use std::io::Write;
    use std::ptr;
    static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn captype_basic() {
        let _g = TEST_MUTEX.lock().unwrap();
        let w = CString::new("vim").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, ptr::null()), 0);
        let w = CString::new("Vim").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, ptr::null()), WF_ONECAP);
        let w = CString::new("VIM").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, ptr::null()), WF_ALLCAP);
        let w = CString::new("vIM").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, ptr::null()), WF_KEEPCAP);
    }

    #[test]
    fn load_and_suggest() {
        let _g = TEST_MUTEX.lock().unwrap();
        let mut path = std::env::temp_dir();
        path.push("dict.txt");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "apple").unwrap();
        writeln!(f, "apply").unwrap();
        writeln!(f, "banana").unwrap();

        let cpath = CString::new(path.to_str().unwrap()).unwrap();
        assert!(rs_spell_load_dict(cpath.as_ptr()));

        let word = CString::new("appl").unwrap();
        let mut len: usize = 0;
        let ptr = rs_spell_suggest(word.as_ptr(), 5, &mut len as *mut usize);
        assert!(!ptr.is_null());
        assert_eq!(len, 2);
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        let mut res: Vec<String> = slice
            .iter()
            .map(|&p| unsafe { CStr::from_ptr(p).to_string_lossy().into_owned() })
            .collect();
        rs_spell_free_suggestions(ptr, len);
        res.sort();
        assert_eq!(res, vec!["apple", "apply"]);
    }

    #[test]
    fn multilingual_and_performance() {
        let _g = TEST_MUTEX.lock().unwrap();
        use std::time::Instant;

        // create a small dictionary with words from multiple languages
        let mut path = std::env::temp_dir();
        path.push("dict_multi.txt");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "apple").unwrap(); // English
        writeln!(f, "bonjour").unwrap(); // French
        writeln!(f, "hola").unwrap(); // Spanish
        writeln!(f, "こんにちは").unwrap(); // Japanese

        let cpath = CString::new(path.to_str().unwrap()).unwrap();
        assert!(rs_spell_load_dict(cpath.as_ptr()));

        // verify suggestions across languages
        let check = |w: &str, expected: &str| {
            let w = CString::new(w).unwrap();
            let mut len: usize = 0;
            let ptr = rs_spell_suggest(w.as_ptr(), 5, &mut len as *mut usize);
            assert!(!ptr.is_null());
            let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
            let mut res: Vec<String> = slice
                .iter()
                .map(|&p| unsafe { CStr::from_ptr(p).to_string_lossy().into_owned() })
                .collect();
            rs_spell_free_suggestions(ptr, len);
            assert!(res.contains(&expected.to_string()));
        };
        check("appl", "apple");
        check("bonjou", "bonjour");
        check("hol", "hola");
        check("こんにちわ", "こんにちは");

        // performance: load a larger dictionary and ensure it loads quickly
        let mut big_path = std::env::temp_dir();
        big_path.push("dict_big.txt");
        let mut f = File::create(&big_path).unwrap();
        for i in 0..10_000 {
            writeln!(f, "word{}", i).unwrap();
        }

        let cpath = CString::new(big_path.to_str().unwrap()).unwrap();
        let start = Instant::now();
        assert!(rs_spell_load_dict(cpath.as_ptr()));
        let elapsed = start.elapsed();
        // basic sanity check: loading should be reasonably fast
        assert!(elapsed.as_secs() < 1, "loading took {:?}", elapsed);
    }
}
