use regex::bytes::RegexBuilder;

#[test]
fn basic_match_and_exec_nl() {
    let re = RegexBuilder::new("foo").build().unwrap();
    assert!(re.is_match(b"foo bar"));
}

#[test]
fn substitution_and_flags() {
    let re = RegexBuilder::new("foo").case_insensitive(true).build().unwrap();
    let result = re.replace(b"Foo bar".as_slice(), b"baz".as_slice());
    assert_eq!(result.as_ref(), b"baz bar");
}

#[test]
fn capture_offsets() {
    let re = RegexBuilder::new("a.c").build().unwrap();
    let m = re.find(b"zabc").unwrap();
    assert_eq!((m.start(), m.end()), (1, 4));
}

#[test]
fn invalid_pattern_returns_err() {
    assert!(RegexBuilder::new("[a-").build().is_err());
}

#[test]
fn non_match_returns_none() {
    let re = RegexBuilder::new("foo").build().unwrap();
    assert!(re.find(b"bar").is_none());
}
