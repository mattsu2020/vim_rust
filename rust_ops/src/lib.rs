#![allow(unused_unsafe, non_snake_case)]

use std::os::raw::{c_int, c_long};

const OPF_LINES: u8 = 1; // operator always works on lines
const OPF_CHANGE: u8 = 2; // operator changes text

const CTRL_A: u8 = 0x01;
const CTRL_X: u8 = 0x18;

const OPCHARS: &[(u8, u8, u8)] = &[
    (0, 0, 0),                             // OP_NOP
    (b'd', 0, OPF_CHANGE),                 // OP_DELETE
    (b'y', 0, 0),                          // OP_YANK
    (b'c', 0, OPF_CHANGE),                 // OP_CHANGE
    (b'<', 0, OPF_LINES | OPF_CHANGE),     // OP_LSHIFT
    (b'>', 0, OPF_LINES | OPF_CHANGE),     // OP_RSHIFT
    (b'!', 0, OPF_LINES | OPF_CHANGE),     // OP_FILTER
    (b'g', b'~', OPF_CHANGE),              // OP_TILDE
    (b'=', 0, OPF_LINES | OPF_CHANGE),     // OP_INDENT
    (b'g', b'q', OPF_LINES | OPF_CHANGE),  // OP_FORMAT
    (b':', 0, OPF_LINES),                  // OP_COLON
    (b'g', b'U', OPF_CHANGE),              // OP_UPPER
    (b'g', b'u', OPF_CHANGE),              // OP_LOWER
    (b'J', 0, OPF_LINES | OPF_CHANGE),     // DO_JOIN
    (b'g', b'J', OPF_LINES | OPF_CHANGE),  // DO_JOIN_NS
    (b'g', b'?', OPF_CHANGE),              // OP_ROT13
    (b'r', 0, OPF_CHANGE),                 // OP_REPLACE
    (b'I', 0, OPF_CHANGE),                 // OP_INSERT
    (b'A', 0, OPF_CHANGE),                 // OP_APPEND
    (b'z', b'f', OPF_LINES),               // OP_FOLD
    (b'z', b'o', OPF_LINES),               // OP_FOLDOPEN
    (b'z', b'O', OPF_LINES),               // OP_FOLDOPENREC
    (b'z', b'c', OPF_LINES),               // OP_FOLDCLOSE
    (b'z', b'C', OPF_LINES),               // OP_FOLDCLOSEREC
    (b'z', b'd', OPF_LINES),               // OP_FOLDDEL
    (b'z', b'D', OPF_LINES),               // OP_FOLDDELREC
    (b'g', b'w', OPF_LINES | OPF_CHANGE),  // OP_FORMAT2
    (b'g', b'@', OPF_CHANGE),              // OP_FUNCTION
    (CTRL_A, 0, OPF_CHANGE),               // OP_NR_ADD
    (CTRL_X, 0, OPF_CHANGE),               // OP_NR_SUB
];

const OP_NOP: usize = 0;
const OP_DELETE: usize = 1;
const OP_YANK: usize = 2;
const OP_LSHIFT: usize = 4;
const OP_TILDE: usize = 7;
const OP_REPLACE: usize = 16;
const OP_NR_ADD: usize = 28;
const OP_NR_SUB: usize = 29;

#[no_mangle]
pub extern "C" fn get_op_type(char1: c_int, char2: c_int) -> c_int {
    if char1 == b'r' as c_int {
        return OP_REPLACE as c_int;
    }
    if char1 == b'~' as c_int {
        return OP_TILDE as c_int;
    }
    if char1 == b'g' as c_int && char2 == CTRL_A as c_int {
        return OP_NR_ADD as c_int;
    }
    if char1 == b'g' as c_int && char2 == CTRL_X as c_int {
        return OP_NR_SUB as c_int;
    }
    if char1 == b'z' as c_int && char2 == b'y' as c_int {
        return OP_YANK as c_int;
    }
    for (i, &(c1, c2, _)) in OPCHARS.iter().enumerate() {
        if c1 as c_int == char1 && c2 as c_int == char2 {
            return i as c_int;
        }
    }
    OP_NOP as c_int
}

#[no_mangle]
pub extern "C" fn rs_op_on_lines(op: c_int) -> c_int {
    if op < 0 || (op as usize) >= OPCHARS.len() {
        return 0;
    }
    if OPCHARS[op as usize].2 & OPF_LINES != 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn op_is_change(op: c_int) -> c_int {
    if op < 0 || (op as usize) >= OPCHARS.len() {
        return 0;
    }
    if OPCHARS[op as usize].2 & OPF_CHANGE != 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn get_op_char(optype: c_int) -> c_int {
    if optype < 0 || (optype as usize) >= OPCHARS.len() {
        return 0;
    }
    OPCHARS[optype as usize].0 as c_int
}

#[no_mangle]
pub extern "C" fn get_extra_op_char(optype: c_int) -> c_int {
    if optype < 0 || (optype as usize) >= OPCHARS.len() {
        return 0;
    }
    OPCHARS[optype as usize].1 as c_int
}

#[repr(C)]
pub struct oparg_T {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct block_def {
    pub startspaces: c_int,
    pub endspaces: c_int,
    pub textlen: c_int,
    pub textstart: *mut u8,
    pub textcol: c_int,
    pub start_vcol: c_int,
    pub end_vcol: c_int,
    pub is_short: c_int,
    pub is_MAX: c_int,
    pub is_oneChar: c_int,
    pub pre_whitesp: c_int,
    pub pre_whitesp_c: c_int,
    pub end_char_vcols: c_int,
    pub start_char_vcols: c_int,
}

impl Default for block_def {
    fn default() -> Self {
        Self {
            startspaces: 0,
            endspaces: 0,
            textlen: 0,
            textstart: std::ptr::null_mut(),
            textcol: 0,
            start_vcol: 0,
            end_vcol: 0,
            is_short: 0,
            is_MAX: 0,
            is_oneChar: 0,
            pre_whitesp: 0,
            pre_whitesp_c: 0,
            end_char_vcols: 0,
            start_char_vcols: 0,
        }
    }
}

#[cfg(not(test))]
extern "C" {
    fn op_shift_c(oap: *mut oparg_T, curs_top: c_int, amount: c_int);
    fn op_delete_c(oap: *mut oparg_T) -> c_int;
    fn op_replace_c(oap: *mut oparg_T, c: c_int) -> c_int;
    fn op_tilde_c(oap: *mut oparg_T);
    fn op_insert_c(oap: *mut oparg_T, count1: c_long);
    fn op_change_c(oap: *mut oparg_T) -> c_int;
    fn op_addsub_c(oap: *mut oparg_T, Prenum1: c_long, g_cmd: c_int);
    fn op_colon_c(oap: *mut oparg_T);
    fn op_function_c(oap: *mut oparg_T);
    fn auto_format(trailwhite: c_int, prev_line: c_int);
}

#[cfg(test)]
#[no_mangle]
extern "C" fn op_shift_c(_oap: *mut oparg_T, _curs_top: c_int, _amount: c_int) {}
#[cfg(test)]
#[no_mangle]
extern "C" fn op_delete_c(_oap: *mut oparg_T) -> c_int { 0 }
#[cfg(test)]
#[no_mangle]
extern "C" fn op_replace_c(_oap: *mut oparg_T, _c: c_int) -> c_int { 0 }
#[cfg(test)]
#[no_mangle]
extern "C" fn op_tilde_c(_oap: *mut oparg_T) {}
#[cfg(test)]
#[no_mangle]
extern "C" fn op_insert_c(_oap: *mut oparg_T, _count1: c_long) {}
#[cfg(test)]
#[no_mangle]
extern "C" fn op_change_c(_oap: *mut oparg_T) -> c_int { 0 }
#[cfg(test)]
#[no_mangle]
extern "C" fn op_addsub_c(_oap: *mut oparg_T, _Prenum1: c_long, _g_cmd: c_int) {}
#[cfg(test)]
#[no_mangle]
extern "C" fn op_colon_c(_oap: *mut oparg_T) {}
#[cfg(test)]
#[no_mangle]
extern "C" fn op_function_c(_oap: *mut oparg_T) {}

#[cfg(test)]
pub(crate) static mut AUTO_COUNT: c_int = 0;
#[cfg(test)]
#[no_mangle]
pub extern "C" fn auto_format(_trailwhite: c_int, _prev_line: c_int) {
    unsafe { AUTO_COUNT += 1; }
}

#[no_mangle]
pub extern "C" fn rs_op_shift(oap: *mut oparg_T, curs_top: c_int, amount: c_int) {
    unsafe { op_shift_c(oap, curs_top, amount) }
}

#[no_mangle]
pub extern "C" fn rs_op_delete(oap: *mut oparg_T) -> c_int {
    unsafe { op_delete_c(oap) }
}

#[no_mangle]
pub extern "C" fn rs_op_replace(oap: *mut oparg_T, c: c_int) -> c_int {
    unsafe { op_replace_c(oap, c) }
}

#[no_mangle]
pub extern "C" fn rs_op_tilde(oap: *mut oparg_T) {
    unsafe { op_tilde_c(oap) }
}

#[no_mangle]
pub extern "C" fn rs_op_insert(oap: *mut oparg_T, count1: c_long) {
    unsafe {
        op_insert_c(oap, count1);
        let cnt = if count1 > 0 { count1 } else { 0 };
        for _ in 0..cnt {
            auto_format(0, 1);
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_op_change(oap: *mut oparg_T) -> c_int {
    unsafe { op_change_c(oap) }
}

#[no_mangle]
pub extern "C" fn rs_op_addsub(oap: *mut oparg_T, Prenum1: c_long, g_cmd: c_int) {
    unsafe { op_addsub_c(oap, Prenum1, g_cmd) }
}

#[no_mangle]
pub extern "C" fn rs_op_colon(oap: *mut oparg_T) {
    unsafe { op_colon_c(oap) }
}

#[no_mangle]
pub extern "C" fn rs_op_function(oap: *mut oparg_T) {
    unsafe { op_function_c(oap) }
}

#[no_mangle]
pub extern "C" fn rs_skip_block_whitespace(
    bd: *mut block_def,
    line: *const u8,
    line_len: usize,
) -> c_int {
    unsafe {
        if bd.is_null() || line.is_null() {
            return 0;
        }
        let bd_ref = &mut *bd;
        let slice = std::slice::from_raw_parts(line, line_len);
        let start_offset = bd_ref.textstart.offset_from(line) as usize;
        if start_offset >= slice.len() {
            bd_ref.textstart = line.add(slice.len()) as *mut u8;
            return 0;
        }
        let mut idx = start_offset;
        while idx < slice.len() && (slice[idx] == b' ' || slice[idx] == b'\t') {
            idx += 1;
        }
        bd_ref.textstart = line.add(idx) as *mut u8;
        let skipped = idx - start_offset;
        bd_ref.start_vcol += skipped as c_int;
        skipped as c_int
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        let res = unsafe { rs_op_change(std::ptr::null_mut()) };
        assert_eq!(res, 0);
    }

    #[test]
    fn get_op_type_basic() {
        assert_eq!(get_op_type(b'd' as c_int, 0), OP_DELETE as c_int);
    }

    #[test]
    fn op_on_lines_checks_flag() {
        assert_eq!(rs_op_on_lines(OP_LSHIFT as c_int), 1);
        assert_eq!(rs_op_on_lines(OP_YANK as c_int), 0);
    }

    #[test]
    fn auto_formats_each_count() {
        unsafe { AUTO_COUNT = 0; }
        rs_op_insert(std::ptr::null_mut(), 3);
        unsafe { assert_eq!(AUTO_COUNT, 3); }
    }

    #[test]
    fn skip_block_whitespace_moves_pointer() {
        let line = b"  abc";
        let mut bd = block_def { textstart: line.as_ptr() as *mut u8, ..Default::default() };
        let skipped = rs_skip_block_whitespace(&mut bd, line.as_ptr(), line.len());
        assert_eq!(skipped, 2);
        unsafe {
            assert_eq!(bd.textstart, line.as_ptr().add(2) as *mut u8);
        }
        assert_eq!(bd.start_vcol, 2);
    }
}
