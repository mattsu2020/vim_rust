use rust_fold::{
    rs_fold_add, rs_fold_find, rs_fold_render, rs_fold_state_free, rs_fold_state_new,
    rs_fold_update,
};
use std::os::raw::c_long;

#[test]
fn regression_add_and_update() {
    let state = rs_fold_state_new();
    rs_fold_add(state, 1, 5, 0, 0);
    rs_fold_add(state, 10, 3, 0, 0);
    assert_eq!(rs_fold_render(state), 8);
    rs_fold_update(state, 0, 1, 4, 0, 0);
    assert_eq!(rs_fold_render(state), 7);
    let mut first: c_long = 0;
    let mut last: c_long = 0;
    assert_eq!(rs_fold_find(state, 2, &mut first, &mut last), 1);
    assert_eq!(first, 1);
    assert_eq!(last, 4);
    unsafe { rs_fold_state_free(state) };
}

#[test]
fn nested_fold_lookup() {
    let state = rs_fold_state_new();
    rs_fold_add(state, 1, 10, 0, 0);
    rs_fold_add(state, 3, 4, 0, 0);
    let mut first: c_long = 0;
    let mut last: c_long = 0;
    assert_eq!(rs_fold_find(state, 4, &mut first, &mut last), 1);
    assert_eq!(first, 3);
    assert_eq!(last, 6);
    unsafe { rs_fold_state_free(state) };
}
