use rust_evalvars::{win_create, win_getid};
use rust_evalwindow::f_win_getid;

#[test]
fn returns_window_id_for_number() {
    win_create();
    win_create();
    let id = f_win_getid(Some(2));
    assert_eq!(id, win_getid(2) as i64);
}

#[test]
fn defaults_to_current_window() {
    win_create();
    let id = f_win_getid(None);
    assert_eq!(id, win_getid(0) as i64);
}
