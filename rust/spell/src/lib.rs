use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::os::raw::{c_char, c_int, c_uchar};
use std::sync::{Mutex, OnceLock};

const WF_ONECAP: c_int = 0x02;
const WF_ALLCAP: c_int = 0x04;
const WF_KEEPCAP: c_int = 0x80;

#[derive(Default)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    end: bool,
}

#[derive(Default)]
struct Trie {
    root: TrieNode,
    words: Vec<String>,
}

impl Trie {
    fn new() -> Self {
        Self::default()
    }

    fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for ch in word.chars() {
            node = node.children.entry(ch).or_insert_with(TrieNode::default);
        }
        if !node.end {
            node.end = true;
            self.words.push(word.to_string());
        }
    }

    fn contains(&self, word: &str) -> bool {
        let mut node = &self.root;
        for ch in word.chars() {
            match node.children.get(&ch) {
                Some(next) => node = next,
                None => return false,
            }
        }
        node.end
    }

    fn load_from_file(&mut self, path: &str) -> bool {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let word = line.trim();
                if !word.is_empty() {
                    self.insert(word);
                }
            }
            true
        } else {
            false
        }
    }

    fn save_to_file(&self, path: &str) -> bool {
        if let Ok(mut file) = File::create(path) {
            for w in &self.words {
                if writeln!(file, "{}", w).is_err() {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    fn suggestions(&self, word: &str, max: usize) -> Vec<String> {
        let mut items: Vec<(usize, &String)> = self
            .words
            .iter()
            .map(|w| (levenshtein(w, word), w))
            .collect();
        items.sort_by_key(|(d, _)| *d);
        items
            .into_iter()
            .take(max)
            .map(|(_, w)| w.clone())
            .collect()
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let mut costs: Vec<usize> = (0..=b.chars().count()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut last = i;
        costs[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let new = if ca == cb { last } else { last + 1 };
            last = costs[j + 1];
            costs[j + 1] = std::cmp::min(std::cmp::min(costs[j] + 1, costs[j + 1] + 1), new);
        }
    }
    costs[b.chars().count()]
}

static TRIE: OnceLock<Mutex<Trie>> = OnceLock::new();

fn trie() -> &'static Mutex<Trie> {
    TRIE.get_or_init(|| Mutex::new(Trie::new()))
}

#[no_mangle]
pub extern "C" fn rs_spell_load_dictionary(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(path) };
    if let Ok(p) = c_str.to_str() {
        let mut trie = trie().lock().unwrap();
        trie.load_from_file(p)
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn rs_spell_check(word: *const c_char) -> bool {
    if word.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(word) };
    if let Ok(w) = c_str.to_str() {
        let trie = trie().lock().unwrap();
        trie.contains(w)
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn rs_spell_best_suggestion(word: *const c_char) -> *mut c_char {
    if word.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(word) };
    if let Ok(w) = c_str.to_str() {
        let trie = trie().lock().unwrap();
        if let Some(sug) = trie.suggestions(w, 1).get(0) {
            if let Ok(c) = CString::new(sug.as_str()) {
                return c.into_raw();
            }
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn rs_spell_free_cstring(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            CString::from_raw(s);
        };
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
        let mut firstcap = first.is_ascii_uppercase();
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
    use std::ffi::CString;

    #[test]
    fn captype_basic() {
        let w = CString::new("vim").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, std::ptr::null()), 0);
        let w = CString::new("Vim").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, std::ptr::null()), WF_ONECAP);
        let w = CString::new("VIM").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, std::ptr::null()), WF_ALLCAP);
        let w = CString::new("vIM").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, std::ptr::null()), WF_KEEPCAP);
    }

    #[test]
    fn multilang_lookup_and_suggest() {
        let mut t = Trie::new();
        t.insert("hello");
        t.insert("hola");
        t.insert("bonjour");
        t.insert("こんにちは");

        assert!(t.contains("hola"));
        assert!(t.contains("こんにちは"));
        assert!(!t.contains("adios"));

        let sug = t.suggestions("bonjor", 2);
        assert!(sug.contains(&"bonjour".to_string()));
    }
}

