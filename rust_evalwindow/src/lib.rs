use rust_evalvars::win_getid;

/// Implementation of the `win_getid()` Vim function in pure Rust.
///
/// `arg` corresponds to the optional window number.  When `None` or `0`, the
/// current window (the first one) is used.
pub fn f_win_getid(arg: Option<i64>) -> i64 {
    let winnr = arg.unwrap_or(0) as i32;
    win_getid(winnr) as i64
}
