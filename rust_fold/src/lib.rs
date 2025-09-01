use std::os::raw::{c_long, c_uchar};

#[derive(Clone, Debug, PartialEq)]
pub struct Fold {
    pub fd_top: c_long,
    pub fd_len: c_long,
    pub fd_flags: c_uchar,
    pub fd_small: c_uchar,
    pub fd_nested: Vec<Fold>,
}

impl Fold {
    fn new(top: c_long, len: c_long, flags: c_uchar, small: c_uchar) -> Self {
        Fold { fd_top: top, fd_len: len, fd_flags: flags, fd_small: small, fd_nested: Vec::new() }
    }
}

pub struct FoldState {
    folds: Vec<Fold>,
}

impl FoldState {
    fn new() -> Self { Self { folds: Vec::new() } }
    fn add_fold(&mut self, top: c_long, len: c_long, flags: c_uchar, small: c_uchar) {
        self.folds.push(Fold::new(top, len, flags, small));
    }
    fn update_fold(&mut self, idx: usize, top: c_long, len: c_long, flags: c_uchar, small: c_uchar) {
        if let Some(f) = self.folds.get_mut(idx) {
            f.fd_top = top;
            f.fd_len = len;
            f.fd_flags = flags;
            f.fd_small = small;
        }
    }
    fn render(&self) -> c_long {
        self.folds.iter().map(|f| f.fd_len).sum()
    }
}

#[no_mangle]
pub extern "C" fn rs_fold_state_new() -> *mut FoldState {
    Box::into_raw(Box::new(FoldState::new()))
}

#[no_mangle]
pub extern "C" fn rs_fold_state_free(state: *mut FoldState) {
    if !state.is_null() {
        unsafe {
            drop(Box::from_raw(state));
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_fold_add(state: *mut FoldState, top: c_long, len: c_long, flags: c_uchar, small: c_uchar) {
    if let Some(s) = unsafe { state.as_mut() } {
        s.add_fold(top, len, flags, small);
    }
}

#[no_mangle]
pub extern "C" fn rs_fold_update(state: *mut FoldState, idx: c_long, top: c_long, len: c_long, flags: c_uchar, small: c_uchar) {
    if let Some(s) = unsafe { state.as_mut() } {
        if idx >= 0 {
            s.update_fold(idx as usize, top, len, flags, small);
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_fold_render(state: *const FoldState) -> c_long {
    unsafe { state.as_ref().map(|s| s.render()).unwrap_or(0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_flow() {
        let mut state = FoldState::new();
        state.add_fold(1, 5, 0, 0);
        state.add_fold(10, 3, 1, 0);
        assert_eq!(state.render(), 8);
        state.update_fold(0, 1, 4, 0, 0);
        assert_eq!(state.render(), 7);
    }
}
