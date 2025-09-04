use rust_spell::{
    rs_spell_check, rs_spell_free_suggestions, rs_spell_load_dict, rs_spell_suggest,
};
use rust_spellfile::{build_from_words, write_spellfile};
use std::ffi::{CStr, CString};

#[test]
fn load_check_and_suggest() {
    let mut path = std::env::temp_dir();
    path.push("dict_test.rspf");
    let data = build_from_words(&["apple", "apply", "banana"]);
    write_spellfile(&path, &data).unwrap();

    let cpath = CString::new(path.to_str().unwrap()).unwrap();
    assert!(rs_spell_load_dict(cpath.as_ptr()));

    let word = CString::new("apple").unwrap();
    assert!(rs_spell_check(word.as_ptr()));
    let missing = CString::new("appl").unwrap();
    assert!(!rs_spell_check(missing.as_ptr()));

    let sugg_word = CString::new("appl").unwrap();
    let mut len: usize = 0;
    let ptr = rs_spell_suggest(sugg_word.as_ptr(), 5, &mut len as *mut usize);
    assert!(!ptr.is_null());
    assert!(len >= 2);
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    let mut res: Vec<String> = slice
        .iter()
        .map(|&p| unsafe { CStr::from_ptr(p).to_string_lossy().into_owned() })
        .collect();
    rs_spell_free_suggestions(ptr, len);
    res.sort();
    assert_eq!(res, vec!["apple", "apply"]);
}
