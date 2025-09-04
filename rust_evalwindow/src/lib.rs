use rust_evalvars::{win_create, win_getid};

/// Implementation of the `win_getid()` Vim function in pure Rust.
///
/// `arg` corresponds to the optional window number.  When `None` or `0`, the
/// current window (the first one) is used.
pub fn f_win_getid(arg: Option<i64>) -> i64 {
    let winnr = arg.unwrap_or(0) as i32;
    win_getid(winnr) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
