use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uchar};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Default)]
pub struct Node {
    pub children: HashMap<char, Node>,
    pub is_word: bool,
}

#[derive(Default)]
pub struct Trie {
    pub root: Node,
}

impl Trie {
    pub fn new() -> Self {
        Self { root: Node::default() }
    }

    pub fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for ch in word.chars() {
            node = node.children.entry(ch).or_default();
        }
        node.is_word = true;
    }

    pub fn contains(&self, word: &str) -> bool {
        let mut node = &self.root;
        for ch in word.chars() {
            match node.children.get(&ch) {
                Some(n) => node = n,
                None => return false,
            }
        }
        node.is_word
    }
}

fn load_dict(path: &str) -> std::io::Result<Trie> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut trie = Trie::new();
    for line in reader.lines() {
        let line = line?;
        let word = line.trim();
        if !word.is_empty() {
            trie.insert(word);
        }
    }
    Ok(trie)
}

fn suggest(trie: &Trie, word: &str, max: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let chars: Vec<char> = word.chars().collect();
    let mut q: VecDeque<(&Node, String, usize, usize)> = VecDeque::new();

    q.push_back((&trie.root, String::new(), 0, 0));

    while let Some((node, prefix, idx, edits)) = q.pop_front() {
        if edits > 1 {
            continue;
        }

        if idx == chars.len() {
            if node.is_word && edits == 1 && seen.insert(prefix.clone()) {
                out.push(prefix.clone());
                if out.len() >= max {
                    return out;
                }
            }
            if edits < 1 {
                for (ch, child) in &node.children {
                    let mut new_pref = prefix.clone();
                    new_pref.push(*ch);
                    q.push_back((child, new_pref, idx, edits + 1));
                }
            }
            continue;
        }

        if edits < 1 {
            // deletion
            q.push_back((node, prefix.clone(), idx + 1, edits + 1));
        }

        if edits < 1 && idx + 1 < chars.len() {
            if let Some(next_node) = node.children.get(&chars[idx + 1]) {
                if let Some(after) = next_node.children.get(&chars[idx]) {
                    let mut new_pref = prefix.clone();
                    new_pref.push(chars[idx + 1]);
                    new_pref.push(chars[idx]);
                    q.push_back((after, new_pref, idx + 2, edits + 1));
                }
            }
        }

        for (ch, child) in &node.children {
            let mut new_pref = prefix.clone();
            new_pref.push(*ch);
            if *ch == chars[idx] {
                q.push_back((child, new_pref.clone(), idx + 1, edits));
                if edits < 1 {
                    // insertion
                    q.push_back((child, new_pref, idx, edits + 1));
                }
            } else if edits < 1 {
                // substitution
                q.push_back((child, new_pref.clone(), idx + 1, edits + 1));
                // insertion
                q.push_back((child, new_pref, idx, edits + 1));
            }
        }

        if out.len() >= max {
            break;
        }
    }

    out
}

const WF_ONECAP: c_int = 0x02;
const WF_ALLCAP: c_int = 0x04;
const WF_KEEPCAP: c_int = 0x80;

static TRIE: OnceLock<Mutex<Trie>> = OnceLock::new();

fn trie() -> Result<MutexGuard<'static, Trie>, PoisonError<MutexGuard<'static, Trie>>> {
    TRIE.get_or_init(|| Mutex::new(Trie::new())).lock()
}

#[no_mangle]
pub extern "C" fn rs_spell_load_dict(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(path) };
    let Ok(p) = cstr.to_str() else {
        return false;
    };
    match load_dict(p) {
        Ok(t) => match trie() {
            Ok(mut guard) => {
                *guard = t;
                true
            }
            Err(_) => false,
        },
        Err(_) => false,
    }
}

#[no_mangle]
pub extern "C" fn rs_spell_check(word: *const c_char) -> bool {
    if word.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(word) };
    let Ok(w) = cstr.to_str() else {
        return false;
    };
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
            unsafe { *len = 0 }; // set length to zero on error
            return std::ptr::null_mut();
        }
    };
    unsafe {
        *len = suggestions.len();
    }
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
        let mut iter = bytes.iter().skip_while(|&&c| !c.is_ascii_alphabetic());
        let Some(&first) = iter.next() else {
            return 0;
        };
        let firstcap = first.is_ascii_uppercase();
        let mut allcap = firstcap;
        let mut past_second = false;
        for &c in iter.filter(|&&c| c.is_ascii_alphabetic()) {
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

    #[test]
    fn trie_lock_error() {
        let _g = TEST_MUTEX.lock().unwrap();

        // Poison the lock
        let _ = std::panic::catch_unwind(|| {
            let _guard = trie().unwrap();
            panic!("poison");
        });

        // prepare valid dictionary path
        let mut path = std::env::temp_dir();
        path.push("dict_poison.txt");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "apple").unwrap();

        let cpath = CString::new(path.to_str().unwrap()).unwrap();
        // loading should fail due to poisoned lock
        assert!(!rs_spell_load_dict(cpath.as_ptr()));

        // suggestions should also fail gracefully
        let word = CString::new("appl").unwrap();
        let mut len: usize = 0;
        let ptr = rs_spell_suggest(word.as_ptr(), 5, &mut len as *mut usize);
        assert!(ptr.is_null());
        assert_eq!(len, 0);

        // clear poison for other tests
        TRIE.get().unwrap().clear_poison();
    }
}
