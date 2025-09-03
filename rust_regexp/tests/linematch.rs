use rust_regexp::line_match;

#[test]
fn distance_basic() {
    assert_eq!(line_match("kitten", "sitting"), 3);
    assert_eq!(line_match("same", "same"), 0);
}
