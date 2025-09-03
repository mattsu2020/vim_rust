use rust_regexp::fuzzy_match;

#[test]
fn fuzz_basic() {
    assert_eq!(fuzzy_match("fz", "fuzzy"), Some(2));
    assert_eq!(fuzzy_match("abc", "acb"), None);
}
