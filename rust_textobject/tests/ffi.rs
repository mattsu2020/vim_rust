use rust_textobject::{skip_chars, Direction};

#[test]
fn skip_chars_integration() {
    let s = "hello world";
    let idx = skip_chars(s, 0, Direction::Forward, false);
    assert_eq!(idx, 5);
}
