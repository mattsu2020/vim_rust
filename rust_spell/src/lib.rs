use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uchar};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use std::collections::{HashSet, VecDeque};

use rust_spellfile::{read_spellfile, SpellFile};
#[cfg(test)]
use rust_spellfile::{build_from_words, write_spellfile};

#[derive(Default)]
pub struct Trie {
    byts: Vec<u8>,
    idxs: Vec<u32>,
}

impl Trie {
    fn from_spellfile(sf: SpellFile) -> Self {
        Self { byts: sf.byts, idxs: sf.idxs }
    }

    fn child(&self, node: usize, ch: u8) -> Option<usize> {
        if node >= self.byts.len() { return None; }
        let len = self.byts[node] as usize;
        let slice = &self.byts[node + 1 .. node + 1 + len];
        if let Ok(pos) = slice.binary_search(&ch) {
            Some(self.idxs[node + 1 + pos] as usize)
        } else {
            None
        }
    }

    fn is_word(&self, node: usize) -> bool {
        if node >= self.byts.len() { return false; }
        let len = self.byts[node] as usize;
        let slice = &self.byts[node + 1 .. node + 1 + len];
        if let Ok(pos) = slice.binary_search(&0) {
            self.idxs[node + 1 + pos] != 0
        } else {
            false
        }
    }

    pub fn contains(&self, word: &str) -> bool {
        let mut idx = 0usize;
        for b in word.bytes() {
            match self.child(idx, b) {
                Some(i) => idx = i,
                None => return false,
            }
        }
        self.is_word(idx)
    }
}

fn load_dict(path: &str) -> std::io::Result<Trie> {
    let data = read_spellfile(path)?;
    Ok(Trie::from_spellfile(data))
}

fn suggest(trie: &Trie, word: &str, max: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let chars: Vec<u8> = word.bytes().collect();
    let mut q: VecDeque<(usize, Vec<u8>, usize, usize)> = VecDeque::new();

    q.push_back((0, Vec::new(), 0, 0));

    while let Some((node, prefix, idx, edits)) = q.pop_front() {
        if edits > 1 { continue; }

        if idx == chars.len() {
            if trie.is_word(node) && edits == 1 && seen.insert(prefix.clone()) {
                if let Ok(s) = String::from_utf8(prefix.clone()) {
                    out.push(s);
                    if out.len() >= max { return out; }
                }
            }
            if edits < 1 {
                let len = trie.byts[node] as usize;
                for k in 0..len {
                    let ch = trie.byts[node + 1 + k];
                    if ch == 0 { continue; }
                    let child = trie.idxs[node + 1 + k] as usize;
                    let mut new_pref = prefix.clone();
                    new_pref.push(ch);
                    q.push_back((child, new_pref, idx, edits + 1));
                }
            }
            continue;
        }

        if edits < 1 {
            q.push_back((node, prefix.clone(), idx + 1, edits + 1));
        }

        if edits < 1 && idx + 1 < chars.len() {
            if let Some(next_node) = trie.child(node, chars[idx + 1]) {
                if let Some(after) = trie.child(next_node, chars[idx]) {
                    let mut new_pref = prefix.clone();
                    new_pref.push(chars[idx + 1]);
                    new_pref.push(chars[idx]);
                    q.push_back((after, new_pref, idx + 2, edits + 1));
                }
            }
        }

        let len = trie.byts[node] as usize;
        for k in 0..len {
            let ch = trie.byts[node + 1 + k];
            if ch == 0 { continue; }
            let child = trie.idxs[node + 1 + k] as usize;
            let mut new_pref = prefix.clone();
            new_pref.push(ch);
            if ch == chars[idx] {
                q.push_back((child, new_pref.clone(), idx + 1, edits));
                if edits < 1 {
                    q.push_back((child, new_pref, idx, edits + 1));
                }
            } else if edits < 1 {
                q.push_back((child, new_pref.clone(), idx + 1, edits + 1));
                q.push_back((child, new_pref, idx, edits + 1));
            }
        }

        if out.len() >= max { break; }
    }

    out
}

const WF_ONECAP: c_int = 0x02;
const WF_ALLCAP: c_int = 0x04;
const WF_KEEPCAP: c_int = 0x80;

static TRIE: OnceLock<Mutex<Trie>> = OnceLock::new();

fn trie() -> Result<MutexGuard<'static, Trie>, PoisonError<MutexGuard<'static, Trie>>> {
    TRIE.get_or_init(|| Mutex::new(Trie::default())).lock()
}

#[no_mangle]
pub extern "C" fn rs_spell_load_dict(path: *const c_char) -> bool {
    if path.is_null() { return false; }
    let cstr = unsafe { CStr::from_ptr(path) };
    let Ok(p) = cstr.to_str() else { return false; };
    match load_dict(p) {
        Ok(t) => match trie() {
            Ok(mut guard) => { *guard = t; true },
            Err(_) => false,
        },
        Err(_) => false,
    }
}

#[no_mangle]
pub extern "C" fn rs_spell_check(word: *const c_char) -> bool {
    if word.is_null() { return false; }
    let cstr = unsafe { CStr::from_ptr(word) };
    let Ok(w) = cstr.to_str() else { return false; };
    match trie() {
        Ok(trie_guard) => trie_guard.contains(w),
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
    let Ok(w) = cstr.to_str() else {
        return std::ptr::null_mut();
    };
    let suggestions = match trie() {
        Ok(trie_guard) => suggest(&*trie_guard, w, max),
        Err(_) => {
            unsafe { *len = 0 }; return std::ptr::null_mut();
        }
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
    if ptr.is_null() { return; }
    unsafe {
        let v = Vec::from_raw_parts(ptr, len, len);
        for p in v { if !p.is_null() { drop(CString::from_raw(p)); } }
    }
}

#[no_mangle]
pub extern "C" fn captype(word: *const c_uchar, end: *const c_uchar) -> c_int {
    if word.is_null() { return 0; }
    unsafe {
        let len = if end.is_null() {
            let mut l = 0; while *word.add(l) != 0 { l += 1; } l
        } else {
            end.offset_from(word) as usize
        };
        let bytes = std::slice::from_raw_parts(word, len);
        let mut iter = bytes.iter().skip_while(|&&c| !c.is_ascii_alphabetic());
        let Some(&first) = iter.next() else { return 0; };
        let firstcap = first.is_ascii_uppercase();
        let mut allcap = firstcap;
        let mut past_second = false;
        for &c in iter.filter(|&&c| c.is_ascii_alphabetic()) {
            if !c.is_ascii_uppercase() {
                if past_second && allcap { return WF_KEEPCAP; }
                allcap = false;
            } else if !allcap {
                return WF_KEEPCAP;
            }
            past_second = true;
        }
        if allcap { WF_ALLCAP } else if firstcap { WF_ONECAP } else { 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::ptr;
    use std::time::Instant;

    static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn write_words(path: &std::path::Path, words: &[&str]) {
        let data = build_from_words(words);
        write_spellfile(path, &data).unwrap();
    }

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
    fn captype_edge_cases() {
        let _g = TEST_MUTEX.lock().unwrap();
        let w = CString::new("123Vim").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, ptr::null()), WF_ONECAP);
        let w = CString::new("123").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, ptr::null()), 0);
    }

    #[test]
    fn load_and_suggest() {
        let _g = TEST_MUTEX.lock().unwrap();
        let mut path = std::env::temp_dir();
        path.push("dict.rspf");
        write_words(&path, &["apple", "apply", "banana"]);

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
        let mut path = std::env::temp_dir();
        path.push("dict_multi.rspf");
        write_words(&path, &["apple", "bonjour", "hola", "こんにちは"]);

        let cpath = CString::new(path.to_str().unwrap()).unwrap();
        assert!(rs_spell_load_dict(cpath.as_ptr()));

        let check = |w: &str, expected: &str| {
            let w = CString::new(w).unwrap();
            let mut len: usize = 0;
            let ptr = rs_spell_suggest(w.as_ptr(), 5, &mut len as *mut usize);
            assert!(!ptr.is_null());
            let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
            let res: Vec<String> = slice
                .iter()
                .map(|&p| unsafe { CStr::from_ptr(p).to_string_lossy().into_owned() })
                .collect();
            rs_spell_free_suggestions(ptr, len);
            assert!(res.contains(&expected.to_string()));
        };
        check("appl", "apple");
        check("bonjou", "bonjour");
        check("hol", "hola");
        let jp = CString::new("こんにちは").unwrap();
        assert!(rs_spell_check(jp.as_ptr()));

        let mut big_path = std::env::temp_dir();
        big_path.push("dict_big.rspf");
        let words: Vec<String> = (0..10_000).map(|i| format!("word{}", i)).collect();
        let word_refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        write_words(&big_path, &word_refs);

        let cpath = CString::new(big_path.to_str().unwrap()).unwrap();
        let start = Instant::now();
        assert!(rs_spell_load_dict(cpath.as_ptr()));
        let elapsed = start.elapsed();
        assert!(elapsed.as_secs() < 1, "loading took {:?}", elapsed);
    }

    #[test]
    fn trie_lock_error() {
        let _g = TEST_MUTEX.lock().unwrap();
        let _ = std::panic::catch_unwind(|| { let _guard = trie().unwrap(); panic!("poison"); });
        let mut path = std::env::temp_dir();
        path.push("dict_poison.rspf");
        write_words(&path, &["apple"]);
        let cpath = CString::new(path.to_str().unwrap()).unwrap();
        assert!(!rs_spell_load_dict(cpath.as_ptr()));
        let word = CString::new("appl").unwrap();
        let mut len: usize = 0;
        let ptr = rs_spell_suggest(word.as_ptr(), 5, &mut len as *mut usize);
        assert!(ptr.is_null());
        assert_eq!(len, 0);
        TRIE.get().unwrap().clear_poison();
    }
}
