use std::os::raw::{c_int, c_long, c_void};

// Simple state tracking for buffer changes.
static mut DID_WARN: bool = false;
static mut CHANGED: bool = false;
static mut READ_ONLY: bool = true;

/// Equivalent of Vim's change_warning().  Returns 1 when a warning should be
/// displayed and 0 otherwise.  Only warns once while the buffer is marked as
/// readonly and not yet changed.
#[no_mangle]
pub extern "C" fn change_warning(_col: c_int) -> c_int {
    unsafe {
        if DID_WARN || CHANGED || !READ_ONLY {
            return 0;
        }
        DID_WARN = true;
    }
    1
}

/// Mark the buffer as changed.  This mirrors Vim's changed() which notifies
/// the editor that modifications have been made.
#[no_mangle]
pub extern "C" fn changed() {
    unsafe {
        CHANGED = true;
    }
}

// ----- Additional stubs to mirror historical change.c API -----

#[no_mangle]
pub extern "C" fn changed_internal() {
    unsafe { CHANGED = true; }
}

#[no_mangle]
pub extern "C" fn f_listener_add(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn f_listener_flush(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn f_listener_remove(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn may_invoke_listeners(
    _buf: *mut c_void,
    _lnum: c_long,
    _lnume: c_long,
    _added: c_int,
) {
}
#[no_mangle]
pub extern "C" fn invoke_listeners(_buf: *mut c_void) {}
#[no_mangle]
pub extern "C" fn remove_listeners(_buf: *mut c_void) {}

#[no_mangle]
pub extern "C" fn changed_bytes(_lnum: c_long, _col: c_int) {
    unsafe { CHANGED = true; }
}
#[no_mangle]
pub extern "C" fn inserted_bytes(_lnum: c_long, _col: c_int, _added: c_int) {
    unsafe { CHANGED = true; }
}
#[no_mangle]
pub extern "C" fn appended_lines(_lnum: c_long, _count: c_long) {
    unsafe { CHANGED = true; }
}
#[no_mangle]
pub extern "C" fn appended_lines_mark(_lnum: c_long, _count: c_long) {}
#[no_mangle]
pub extern "C" fn deleted_lines(_lnum: c_long, _count: c_long) {
    unsafe { CHANGED = true; }
}
#[no_mangle]
pub extern "C" fn deleted_lines_mark(_lnum: c_long, _count: c_long) {}
#[no_mangle]
pub extern "C" fn changed_lines_buf(
    _buf: *mut c_void,
    _lnum: c_long,
    _lnume: c_long,
    _xtra: c_long,
) {
    unsafe { CHANGED = true; }
}
#[no_mangle]
pub extern "C" fn changed_lines(_lnum: c_long, _col: c_int, _lnume: c_long, _xtra: c_long) {
    unsafe { CHANGED = true; }
}
#[no_mangle]
pub extern "C" fn unchanged(_buf: *mut c_void, _ff: c_int, _always_inc_changedtick: c_int) {
    unsafe { CHANGED = false; }
}
#[no_mangle]
pub extern "C" fn save_file_ff(_buf: *mut c_void) {}
#[no_mangle]
pub extern "C" fn file_ff_differs(_buf: *mut c_void, _ignore_empty: c_int) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn ins_bytes(_p: *mut u8) {}
#[no_mangle]
pub extern "C" fn ins_bytes_len(_p: *mut u8, _len: c_int) {}
#[no_mangle]
pub extern "C" fn ins_char(_c: c_int) {}
#[no_mangle]
pub extern "C" fn ins_char_bytes(_buf: *mut u8, _charlen: c_int) {}
#[no_mangle]
pub extern "C" fn ins_str(_s: *mut u8, _slen: usize) {}

const OK: c_int = 1;
#[no_mangle]
pub extern "C" fn del_char(_fixpos: c_int) -> c_int { OK }
#[no_mangle]
pub extern "C" fn del_chars(_count: c_long, _fixpos: c_int) -> c_int { OK }
#[no_mangle]
pub extern "C" fn del_bytes(_count: c_long, _fixpos_arg: c_int, _use_delcombine: c_int) -> c_int { OK }
#[no_mangle]
pub extern "C" fn open_line(
    _dir: c_int,
    _flags: c_int,
    _second_line_indent: c_int,
    did_do_comment: *mut c_int,
) -> c_int {
    unsafe {
        if !did_do_comment.is_null() {
            *did_do_comment = 0;
        }
    }
    OK
}
#[no_mangle]
pub extern "C" fn truncate_line(_fixpos: c_int) -> c_int { OK }
#[no_mangle]
pub extern "C" fn del_lines(_nlines: c_long, _undo: c_int) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warns_only_once() {
        unsafe {
            DID_WARN = false;
            CHANGED = false;
            READ_ONLY = true;
        }
        assert_eq!(change_warning(0), 1);
        assert_eq!(change_warning(0), 0);
    }
}
