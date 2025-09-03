use std::os::raw::{c_int, c_long, c_uchar};

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
        Fold {
            fd_top: top,
            fd_len: len,
            fd_flags: flags,
            fd_small: small,
            fd_nested: Vec::new(),
        }
    }

    fn contains(&self, lnum: c_long) -> bool {
        lnum >= self.fd_top && lnum < self.fd_top + self.fd_len
    }

    fn add_nested(&mut self, top: c_long, len: c_long, flags: c_uchar, small: c_uchar) {
        if let Some(idx) = self.fd_nested.iter().position(|f| f.contains(top)) {
            self.fd_nested[idx].add_nested(top, len, flags, small);
        } else {
            self.fd_nested.push(Fold::new(top, len, flags, small));
            self.fd_nested.sort_by_key(|f| f.fd_top);
        }
    }

    fn find(&self, lnum: c_long) -> Option<(c_long, c_long)> {
        for child in &self.fd_nested {
            if child.contains(lnum) {
                return child.find(lnum);
            }
        }
        if self.contains(lnum) {
            Some((self.fd_top, self.fd_top + self.fd_len - 1))
        } else {
            None
        }
    }

    fn total_len(&self) -> c_long {
        self.fd_len
            + self
                .fd_nested
                .iter()
                .map(|c| c.total_len())
                .sum::<c_long>()
    }
}

pub struct FoldState {
    folds: Vec<Fold>,
}

impl FoldState {
    fn new() -> Self {
        Self { folds: Vec::new() }
    }

    fn add_fold(&mut self, top: c_long, len: c_long, flags: c_uchar, small: c_uchar) {
        if let Some(idx) = self.folds.iter().position(|f| f.contains(top)) {
            let parent = &mut self.folds[idx];
            parent.add_nested(top, len, flags, small);
        } else {
            self.folds.push(Fold::new(top, len, flags, small));
            self.folds.sort_by_key(|f| f.fd_top);
        }
    }

    fn update_fold(
        &mut self,
        idx: usize,
        top: c_long,
        len: c_long,
        flags: c_uchar,
        small: c_uchar,
    ) {
        if let Some(f) = self.folds.get_mut(idx) {
            f.fd_top = top;
            f.fd_len = len;
            f.fd_flags = flags;
            f.fd_small = small;
        }
    }

    fn render(&self) -> c_long {
        self.folds.iter().map(|f| f.total_len()).sum()
    }

    fn find_fold(&self, lnum: c_long) -> Option<(c_long, c_long)> {
        for f in &self.folds {
            if f.contains(lnum) {
                return f.find(lnum);
            }
        }
        None
    }

    fn has_any(&self) -> bool {
        !self.folds.is_empty()
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

#[no_mangle]
pub extern "C" fn rs_fold_has_any(state: *const FoldState) -> c_int {
    unsafe { state.as_ref().map(|s| s.has_any() as c_int).unwrap_or(0) }
}

#[no_mangle]
pub extern "C" fn rs_fold_find(
    state: *const FoldState,
    lnum: c_long,
    firstp: *mut c_long,
    lastp: *mut c_long,
) -> c_int {
    if let Some(s) = unsafe { state.as_ref() } {
        if let Some((first, last)) = s.find_fold(lnum) {
            unsafe {
                if !firstp.is_null() {
                    *firstp = first;
                }
                if !lastp.is_null() {
                    *lastp = last;
                }
            }
            return 1;
        }
    }
    0
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

    #[test]
    fn nested_lookup() {
        let mut state = FoldState::new();
        state.add_fold(1, 10, 0, 0);
        state.add_fold(3, 4, 0, 0); // nested inside first fold
        let (first, last) = state.find_fold(4).unwrap();
        assert_eq!(first, 3);
        assert_eq!(last, 6);
    }

    #[test]
    fn ffi_find_fold() {
        let state = rs_fold_state_new();
        rs_fold_add(state, 1, 5, 0, 0);
        let mut first = 0;
        let mut last = 0;
        let res = rs_fold_find(state, 3, &mut first, &mut last);
        assert_eq!(res, 1);
        assert_eq!(first, 1);
        assert_eq!(last, 5);
        unsafe { rs_fold_state_free(state) };
    }
}
