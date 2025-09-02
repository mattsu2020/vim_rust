use libc::{c_char, c_int, c_long, c_uchar, size_t};
use regex::Regex;
use std::ffi::CStr;

#[repr(C)]
pub struct win_T {
    _private: [u8; 0],
}

#[repr(C)]
pub struct memline_T {
    pub ml_line_count: c_long,
}

#[repr(C)]
pub struct buf_T {
    pub b_ml: memline_T,
}

#[repr(C)]
pub struct searchit_arg_T {
    _private: [u8; 0],
}

#[repr(C)]
pub struct pos_T {
    pub lnum: c_long,
    pub col: c_int,
    pub coladd: c_int,
}

extern "C" {
    fn ml_get_buf(buf: *mut buf_T, lnum: c_long, will_change: c_int) -> *mut c_uchar;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: c_long) -> c_int;
}

#[no_mangle]
pub extern "C" fn rust_searchit(
    _win: *mut win_T,
    buf: *mut buf_T,
    pos: *mut pos_T,
    end_pos: *mut pos_T,
    dir: c_int,
    pat: *const c_uchar,
    patlen: size_t,
    count: c_long,
    _options: c_int,
    _pat_use: c_int,
    _extra_arg: *mut searchit_arg_T,
) -> c_int {
    if pos.is_null() || buf.is_null() || pat.is_null() || patlen == 0 {
        return 0;
    }

    let pat_slice = unsafe { std::slice::from_raw_parts(pat, patlen) };
    let pattern = String::from_utf8_lossy(pat_slice);
    let re = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return 0,
    };

    unsafe {
        let mut lnum = (*pos).lnum;
        let mut col = (*pos).col as usize;
        let line_count = (*buf).b_ml.ml_line_count;
        let mut matched = 0i64;

        if dir >= 0 {
            while lnum <= line_count {
                let line_ptr = ml_get_buf(buf, lnum, 0);
                if line_ptr.is_null() {
                    break;
                }
                let len = ml_get_buf_len(buf, lnum) as usize;
                let slice = std::slice::from_raw_parts(line_ptr, len);
                let text = String::from_utf8_lossy(slice);
                let start = if lnum == (*pos).lnum { col } else { 0 };
                if let Some(m) = re.find(&text[start..]) {
                    matched += 1;
                    if matched >= count as i64 {
                        (*pos).lnum = lnum;
                        (*pos).col = (start + m.start()) as c_int;
                        if !end_pos.is_null() {
                            (*end_pos).lnum = lnum;
                            (*end_pos).col = (start + m.end()) as c_int;
                        }
                        return 1;
                    }
                }
                lnum += 1;
                col = 0;
            }
        } else {
            if lnum > line_count {
                lnum = line_count;
            }
            while lnum >= 1 {
                let line_ptr = ml_get_buf(buf, lnum, 0);
                if line_ptr.is_null() {
                    break;
                }
                let len = ml_get_buf_len(buf, lnum) as usize;
                let slice = std::slice::from_raw_parts(line_ptr, len);
                let text = String::from_utf8_lossy(slice);
                let end = if lnum == (*pos).lnum { col.min(text.len()) } else { text.len() };
                if let Some(m) = re.find_iter(&text[..end]).last() {
                    matched += 1;
                    if matched >= count as i64 {
                        (*pos).lnum = lnum;
                        (*pos).col = m.start() as c_int;
                        if !end_pos.is_null() {
                            (*end_pos).lnum = lnum;
                            (*end_pos).col = m.end() as c_int;
                        }
                        return 1;
                    }
                }
                if lnum == 1 {
                    break;
                }
                lnum -= 1;
                col = usize::MAX;
            }
        }
    }

    0
}

#[no_mangle]
pub extern "C" fn rust_find_pattern_in_path(
    ptr: *mut c_uchar,
    _dir: c_int,
    _len: c_int,
    _whole: c_int,
    _skip_comments: c_int,
    _typ: c_int,
    _count: c_long,
    _action: c_int,
    _start_lnum: c_long,
    _end_lnum: c_long,
    _forceit: c_int,
    _silent: c_int,
) {
    if ptr.is_null() {
        return;
    }
    // Currently a stub: just read the pattern to ensure pointer validity.
    let _ = unsafe { CStr::from_ptr(ptr as *const c_char).to_string_lossy() };
}

