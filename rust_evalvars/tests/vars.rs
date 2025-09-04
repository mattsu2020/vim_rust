use rust_evalvars::{
    get_vim_var_nr, get_vim_var_str, set_vim_var_nr, set_vim_var_str, win_create, win_getid,
};

#[test]
fn var_numbers_roundtrip() {
    assert_eq!(get_vim_var_nr(1), None);
    set_vim_var_nr(1, 42);
    assert_eq!(get_vim_var_nr(1), Some(42));
}

#[test]
fn var_strings_roundtrip() {
    assert_eq!(get_vim_var_str(1), None);
    set_vim_var_str(1, "hello");
    assert_eq!(get_vim_var_str(1), Some("hello".to_string()));
}

#[test]
fn window_ids() {
    let id1 = win_create();
    let id2 = win_create();
    assert_eq!(win_getid(0), id1);
    assert_eq!(win_getid(2), id2);
    assert_eq!(win_getid(3), 0);
}
