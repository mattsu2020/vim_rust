use rust_spell::{rs_spell_check, rs_spell_load_dict};
use rust_spellfile::{build_from_words, write_spellfile};
use std::ffi::CString;

#[test]
fn load_and_reload_dictionaries() {
    let mut path_en = std::env::temp_dir();
    path_en.push("dict_en.rspf");
    let data_en = build_from_words(&["hello", "world"]);
    write_spellfile(&path_en, &data_en).unwrap();
    let cpath_en = CString::new(path_en.to_str().unwrap()).unwrap();
    assert!(rs_spell_load_dict(cpath_en.as_ptr()));
    let hello = CString::new("hello").unwrap();
    let konnichiwa = CString::new("こんにちは").unwrap();
    assert!(rs_spell_check(hello.as_ptr()));
    assert!(!rs_spell_check(konnichiwa.as_ptr()));

    let mut path_jp = std::env::temp_dir();
    path_jp.push("dict_jp.rspf");
    let data_jp = build_from_words(&["こんにちは", "さようなら"]);
    write_spellfile(&path_jp, &data_jp).unwrap();
    let cpath_jp = CString::new(path_jp.to_str().unwrap()).unwrap();
    assert!(rs_spell_load_dict(cpath_jp.as_ptr()));
    assert!(rs_spell_check(konnichiwa.as_ptr()));
    assert!(!rs_spell_check(hello.as_ptr()));
}
