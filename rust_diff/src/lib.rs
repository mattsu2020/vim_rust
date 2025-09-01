use std::os::raw::{c_int, c_long};
use std::ptr;

pub type linenr_T = c_long;

#[repr(C)]
pub struct buf_T {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct tabpage_T {
    pub tp_next: *mut tabpage_T,
    // actual struct has more fields, but we only need tp_next here
    // and treat the rest as opaque
    _rest: [u8; 0],
}

#[repr(C)]
pub struct win_T {
    _unused: [u8; 0],
}

const DB_COUNT: c_int = 8;
const DIFF_FILLER: c_int = 0x001;

extern "C" {
    static mut first_tabpage: *mut tabpage_T;
    static mut curbuf: *mut buf_T;
    pub fn diff_buf_idx_tp(buf: *mut buf_T, tp: *mut tabpage_T) -> c_int;
    pub fn diff_mark_adjust_tp(tp: *mut tabpage_T, idx: c_int,
        line1: linenr_T, line2: linenr_T, amount: c_long, amount_after: c_long);
    static mut diff_flags: c_int;
    pub fn diff_check_with_linestatus(wp: *mut win_T, lnum: linenr_T,
        linestatus: *mut c_int) -> c_int;
}

#[no_mangle]
pub extern "C" fn diff_mark_adjust(line1: linenr_T, line2: linenr_T,
    amount: c_long, amount_after: c_long) {
    unsafe {
        let mut tp = first_tabpage;
        while !tp.is_null() {
            let idx = diff_buf_idx_tp(curbuf, tp);
            if idx != DB_COUNT {
                diff_mark_adjust_tp(tp, idx, line1, line2, amount, amount_after);
            }
            tp = (*tp).tp_next;
        }
    }
}

#[no_mangle]
pub extern "C" fn diff_check_fill(wp: *mut win_T, lnum: linenr_T) -> c_int {
    unsafe {
        if (diff_flags & DIFF_FILLER) == 0 {
            return 0;
        }
        let n = diff_check_with_linestatus(wp, lnum, ptr::null_mut());
        if n <= 0 { 0 } else { n }
    }
}
