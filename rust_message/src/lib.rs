use libc::{c_char, c_int, malloc, free};
use std::ffi::CStr;
use std::io::{self, Write};
use std::ptr;

#[repr(C)]
pub struct MsgHist {
    pub next: *mut MsgHist,
    pub msg: *mut c_char,
    pub attr: c_int,
}

#[cfg(not(test))]
extern "C" {
    static mut first_msg_hist: *mut MsgHist;
    static mut last_msg_hist: *mut MsgHist;
    static mut msg_hist_len: c_int;
    static mut msg_hist_max: c_int;
}

#[cfg(test)]
#[no_mangle]
pub static mut first_msg_hist: *mut MsgHist = ptr::null_mut();
#[cfg(test)]
#[no_mangle]
pub static mut last_msg_hist: *mut MsgHist = ptr::null_mut();
#[cfg(test)]
#[no_mangle]
pub static mut msg_hist_len: c_int = 0;
#[cfg(test)]
#[no_mangle]
pub static mut msg_hist_max: c_int = 500;

unsafe fn alloc_msg_hist() -> *mut MsgHist {
    let size = std::mem::size_of::<MsgHist>();
    let p = malloc(size) as *mut MsgHist;
    if p.is_null() {
        return ptr::null_mut();
    }
    (*p).next = ptr::null_mut();
    (*p).msg = ptr::null_mut();
    (*p).attr = 0;
    p
}

fn format_with_attr(slice: &[u8], attr: c_int) -> Vec<u8> {
    let mut out = Vec::new();
    if attr > 0 {
        out.extend_from_slice(format!("\x1b[{}m", attr).as_bytes());
    }
    out.extend_from_slice(slice);
    if attr > 0 {
        out.extend_from_slice(b"\x1b[0m");
    }
    out.push(0);
    out
}

#[no_mangle]
pub unsafe extern "C" fn rs_add_msg_hist(s: *const c_char, len: c_int, attr: c_int) {
    if s.is_null() {
        return;
    }
    let mut l = len;
    let bytes: &[u8];
    if l < 0 {
        let cstr = CStr::from_ptr(s);
        bytes = cstr.to_bytes();
        l = bytes.len() as c_int;
    } else {
        bytes = std::slice::from_raw_parts(s as *const u8, l as usize);
    }
    let mut start = 0usize;
    let mut end = bytes.len();
    while start < end && bytes[start] == b'\n' { start += 1; }
    while end > start && bytes[end - 1] == b'\n' { end -= 1; }
    if start >= end {
        return;
    }
    let slice = &bytes[start..end];
    let formatted = format_with_attr(slice, attr);
    let msg_ptr = malloc(formatted.len()) as *mut c_char;
    if msg_ptr.is_null() {
        return;
    }
    ptr::copy_nonoverlapping(formatted.as_ptr() as *const c_char, msg_ptr, formatted.len());
    let node = alloc_msg_hist();
    if node.is_null() {
        free(msg_ptr as *mut _);
        return;
    }
    (*node).msg = msg_ptr;
    (*node).attr = attr;
    if !last_msg_hist.is_null() {
        (*last_msg_hist).next = node;
    }
    last_msg_hist = node;
    if first_msg_hist.is_null() {
        first_msg_hist = node;
    }
    msg_hist_len += 1;
    rs_check_msg_hist();
}

#[no_mangle]
pub unsafe extern "C" fn rs_delete_first_msg() -> c_int {
    if msg_hist_len <= 0 || first_msg_hist.is_null() {
        return 1; // FAIL
    }
    let p = first_msg_hist;
    first_msg_hist = (*p).next;
    if first_msg_hist.is_null() {
        last_msg_hist = ptr::null_mut();
    }
    if !(*p).msg.is_null() {
        free((*p).msg as *mut _);
    }
    free(p as *mut _);
    msg_hist_len -= 1;
    0 // OK
}

#[no_mangle]
pub unsafe extern "C" fn rs_check_msg_hist() {
    while msg_hist_len > 0 && msg_hist_len > msg_hist_max {
        rs_delete_first_msg();
    }
}

#[no_mangle]
pub unsafe extern "C" fn rs_render_msg(s: *const c_char, attr: c_int) {
    if s.is_null() {
        return;
    }
    let cstr = CStr::from_ptr(s);
    let bytes = cstr.to_bytes();
    let formatted = format_with_attr(bytes, attr);
    let _ = io::stdout().write_all(&formatted[..formatted.len() - 1]);
    let _ = io::stdout().flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn multibyte_and_color() {
        unsafe {
            // reset globals
            first_msg_hist = ptr::null_mut();
            last_msg_hist = ptr::null_mut();
            msg_hist_len = 0;
            msg_hist_max = 10;

            rs_add_msg_hist(b"\xC3\x84\xC3\x96\xC3\x9C\xC3\xA4\0".as_ptr() as *const c_char, -1, 0);
            assert_eq!(msg_hist_len, 1);
            let msg1 = CStr::from_ptr((*first_msg_hist).msg).to_str().unwrap();
            assert_eq!(msg1, "ÄÖÜä");

            rs_add_msg_hist(b"hello\0".as_ptr() as *const c_char, -1, 31);
            assert_eq!(msg_hist_len, 2);
            let msg2 = CStr::from_ptr((*last_msg_hist).msg).to_str().unwrap();
            assert_eq!(msg2, "\u{1b}[31mhello\u{1b}[0m");
        }
    }
}
