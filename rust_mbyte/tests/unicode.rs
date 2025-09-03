use rust_mbyte::{
    rust_mb_charlen, rust_utf_islower, rust_utf_isupper, rust_utf_ptr2len, rust_utf_tolower,
    rust_utf_toupper,
};
use std::ffi::CString;
use std::os::raw::{c_char, c_int};

#[test]
fn mb_charlen_counts_graphemes() {
    let s = CString::new("e\u{301}o").unwrap();
    assert_eq!(rust_mb_charlen(s.as_ptr() as *const c_char), 2);

    let emoji = CString::new("😀").unwrap();
    assert_eq!(rust_mb_charlen(emoji.as_ptr() as *const c_char), 1);
}

#[test]
fn utf_ptr2len_handles_composites_and_emoji() {
    let s = CString::new("e\u{301}b").unwrap();
    assert_eq!(rust_utf_ptr2len(s.as_ptr() as *const c_char), 3); // "e" + combining accent

    let emoji = CString::new("😀a").unwrap();
    assert_eq!(rust_utf_ptr2len(emoji.as_ptr() as *const c_char), 4); // emoji is 4 bytes
}

#[test]
fn utf_case_conversion_and_checks() {
    assert_eq!(rust_utf_toupper('é' as c_int), 'É' as c_int);
    assert_eq!(rust_utf_tolower('İ' as c_int), 'i' as c_int);
    assert_eq!(rust_utf_toupper('\u{10437}' as c_int), '\u{1040F}' as c_int); // 𐐷 -> 𐐏

    assert_eq!(rust_utf_isupper('É' as c_int), 1);
    assert_eq!(rust_utf_islower('é' as c_int), 1);
    assert_eq!(rust_utf_isupper('\u{1040F}' as c_int), 1); // 𐐏
    assert_eq!(rust_utf_islower('\u{10437}' as c_int), 1); // 𐐷
}
