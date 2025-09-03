use rust_textprop::{add_prop_type, find_prop_type_id, clear_all};

#[test]
fn roundtrip() {
    clear_all();
    let tp = add_prop_type("type1");
    assert!(tp.id > 0);
    assert_eq!(find_prop_type_id("type1"), Some(tp.id));
}
