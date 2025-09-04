use libc::{c_int, c_long, c_void, c_uchar};

// Opaque type aliases for C structs
pub type win_T = c_void;
pub type textprop_T = c_void;
pub type spellvars_T = c_void;
pub type linenr_T = c_long;
pub type char_u = c_uchar;

#[no_mangle]
pub extern "C" fn text_prop_position(
    _wp: *mut win_T,
    _tp: *mut textprop_T,
    _vcol: c_int,
    _scr_col: c_int,
    _n_extra: *mut c_int,
    _p_extra: *mut *mut char_u,
    _n_attr: *mut c_int,
    _n_attr_skip: *mut c_int,
    _do_skip: c_int,
) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn win_line(
    _wp: *mut win_T,
    _lnum: linenr_T,
    _startrow: c_int,
    _endrow: c_int,
    _number_only: c_int,
    _spv: *mut spellvars_T,
) -> c_int {
    0
}
