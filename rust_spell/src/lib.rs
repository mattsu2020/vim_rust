use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::raw::{c_char, c_int, c_uchar};
use std::sync::{Mutex, OnceLock};

// Constants matching Vim's spell checking flags
const WF_ONECAP: c_int = 0x02;
const WF_ALLCAP: c_int = 0x04;
const WF_KEEPCAP: c_int = 0x80;

#[derive(Default)]
struct Node {
    children: HashMap<char, Node>,
    is_word: bool,
}

#[derive(Default)]
struct Trie {
    root: Node,
}

impl Trie {
    fn new() -> Self {
        Self { root: Node::default() }
    }

    fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for ch in word.chars() {
            node = node.children.entry(ch).or_default();
        }
        node.is_word = true;
    }

    fn collect(node: &Node, prefix: &mut String, out: &mut Vec<String>) {
        if node.is_word {
            out.push(prefix.clone());
        }
        for (ch, child) in &node.children {
            prefix.push(*ch);
            Self::collect(child, prefix, out);
            prefix.pop();
        }
    }

    fn all_words(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut prefix = String::new();
        Self::collect(&self.root, &mut prefix, &mut out);
        out
    }

    fn suggest(&self, word: &str, max: usize) -> Vec<String> {
        let mut res = Vec::new();
        for w in self.all_words() {
            if edit_distance_one(word, &w) {
                res.push(w);
                if res.len() >= max {
                    break;
                }
            }
        }
        res
    }
}

fn edit_distance_one(a: &str, b: &str) -> bool {
    if a == b {
        return false;
    }
    let la = a.chars().count();
    let lb = b.chars().count();
    if la.abs_diff(lb) > 1 {
        return false;
    }
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut i = 0usize;
    let mut j = 0usize;
    let mut diff = 0usize;
    while i < la && j < lb {
        if a_chars[i] == b_chars[j] {
            i += 1;
            j += 1;
        } else {
            diff += 1;
            if diff > 1 {
                return false;
            }
            if la > lb {
                i += 1;
            } else if lb > la {
                j += 1;
            } else {
                i += 1;
                j += 1;
            }
        }
    }
    diff += la - i + lb - j;
    diff <= 1
}

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
    let Ok(p) = cstr.to_str() else { return false };
    let file = match File::open(p) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let reader = BufReader::new(file);
    let mut trie = trie().lock().unwrap();
    for line in reader.lines() {
        if let Ok(word) = line {
            let w = word.trim();
            if !w.is_empty() {
                trie.insert(w);
            }
        }
    }
    true
}

#[no_mangle]
pub extern "C" fn rs_spell_suggest(
    word: *const c_char,
    max: usize,
    out_len: *mut usize,
) -> *mut *mut c_char {
    if word.is_null() || out_len.is_null() {
        return std::ptr::null_mut();
    }
    let cstr = unsafe { CStr::from_ptr(word) };
    let Ok(w) = cstr.to_str() else { return std::ptr::null_mut() };
    let trie = trie().lock().unwrap();
    let suggestions = trie.suggest(w, max);
    let len = suggestions.len();
    unsafe { *out_len = len; }
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
    use std::io::Write;
    use std::ptr;

    #[test]
    fn captype_basic() {
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
}
