use rust_spell::{rs_spell_check, rs_spell_load_dict};
use std::ffi::CString;

#[test]
fn load_and_reload_dictionaries() {
    let mut path_en = std::env::temp_dir();
    path_en.push("dict_en.txt");
    std::fs::write(&path_en, b"hello\nworld\n").unwrap();
    let cpath_en = CString::new(path_en.to_str().unwrap()).unwrap();
    assert!(rs_spell_load_dict(cpath_en.as_ptr()));
    let hello = CString::new("hello").unwrap();
    let konnichiwa = CString::new("こんにちは").unwrap();
    assert!(rs_spell_check(hello.as_ptr()));
    assert!(!rs_spell_check(konnichiwa.as_ptr()));

    let mut path_jp = std::env::temp_dir();
    path_jp.push("dict_jp.txt");
    std::fs::write(&path_jp, "こんにちは\nさようなら\n").unwrap();
    let cpath_jp = CString::new(path_jp.to_str().unwrap()).unwrap();
    assert!(rs_spell_load_dict(cpath_jp.as_ptr()));
    assert!(rs_spell_check(konnichiwa.as_ptr()));
    assert!(!rs_spell_check(hello.as_ptr()));
}
