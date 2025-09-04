#[derive(Debug)]
pub struct VTerm {
    cursor: VTermPos,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VTermPos {
    pub row: i32,
    pub col: i32,
}

pub struct VTermState {
    vt: *mut VTerm,
}

pub fn vterm_new(_rows: i32, _cols: i32) -> *mut VTerm {
    Box::into_raw(Box::new(VTerm {
        cursor: VTermPos::default(),
    }))
}

/// # Safety
/// `vt` must be a valid pointer returned by `vterm_new`.
pub unsafe fn vterm_free(vt: *mut VTerm) {
    if !vt.is_null() {
        drop(Box::from_raw(vt));
    }
}

/// # Safety
/// ` _vt` must be a valid pointer.
pub unsafe fn vterm_set_size(_vt: *mut VTerm, _rows: i32, _cols: i32) {}

/// # Safety
/// `vt` must be a valid pointer returned by `vterm_new`.
pub unsafe fn vterm_obtain_state(vt: *mut VTerm) -> *mut VTermState {
    Box::into_raw(Box::new(VTermState { vt }))
}

/// # Safety
/// `state` must be a valid pointer returned by `vterm_obtain_state`.
pub unsafe fn vterm_state_reset(state: *mut VTermState, _hard: i32) {
    if let Some(s) = state.as_mut() {
        if let Some(vt) = s.vt.as_mut() {
            vt.cursor = VTermPos::default();
        }
    }
}

/// # Safety
/// `vt` must be a valid pointer; `_bytes` must point to at least `len` bytes.
pub unsafe fn vterm_input_write(vt: *mut VTerm, _bytes: *const i8, len: usize) -> usize {
    if let Some(v) = vt.as_mut() {
        v.cursor.col += len as i32;
    }
    len
}

/// # Safety
/// `state` and `pos` must be valid pointers.
pub unsafe fn vterm_state_get_cursorpos(state: *const VTermState, pos: *mut VTermPos) {
    if let (Some(s), Some(p)) = (state.as_ref(), pos.as_mut()) {
        if let Some(vt) = s.vt.as_ref() {
            *p = vt.cursor;
        }
    }
}
