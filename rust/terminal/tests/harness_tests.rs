use vim_terminal::harness::TestHarness;

#[test]
fn push_and_retrieve_scrollback() {
    let mut h = TestHarness::new(80, 24).expect("harness");
    // hex for "hello"
    h.push_hex("68 65 6c 6c 6f").expect("push");
    assert_eq!(h.scrollback_line(0).as_deref(), Some("hello"));
}
