use libc::{c_int, c_long, c_void};

pub type oparg_T = c_void;
pub type linenr_T = c_long;

#[no_mangle]
pub extern "C" fn has_format_option(_x: c_int) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn internal_format(
    _textwidth: c_int,
    _second_indent: c_int,
    _flags: c_int,
    _format_only: c_int,
    _c: c_int,
) {
}

#[no_mangle]
pub extern "C" fn auto_format(_trailblank: c_int, _prev_line: c_int) {}

#[no_mangle]
pub extern "C" fn check_auto_format(_end_insert: c_int) {}

#[no_mangle]
pub extern "C" fn comp_textwidth(_ff: c_int) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn op_format(_oap: *mut oparg_T, _keep_cursor: c_int) {}

#[no_mangle]
pub extern "C" fn op_formatexpr(_oap: *mut oparg_T) {}

#[no_mangle]
pub extern "C" fn fex_format(_lnum: linenr_T, _count: c_long, _c: c_int) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn format_lines(_line_count: linenr_T, _avoid_fex: c_int) {}
