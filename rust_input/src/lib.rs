use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};

pub struct InputContext {
    input: VecDeque<u32>,
    redo: VecDeque<u32>,
    record: Vec<u32>,
}

impl InputContext {
    fn new() -> Self {
        Self {
            input: VecDeque::new(),
            redo: VecDeque::new(),
            record: Vec::new(),
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_input_context_new() -> *mut InputContext {
    Box::into_raw(Box::new(InputContext::new()))
}

#[no_mangle]
pub extern "C" fn rs_input_context_free(ptr: *mut InputContext) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_input_feed(ptr: *mut InputContext, key: c_uint) {
    if let Some(ctx) = unsafe { ptr.as_mut() } {
        ctx.input.push_back(key as u32);
        ctx.record.push(key as u32);
    }
}

#[no_mangle]
pub extern "C" fn rs_input_feed_str(ptr: *mut InputContext, s: *const c_char) {
    if ptr.is_null() || s.is_null() {
        return;
    }
    let ctx = unsafe { &mut *ptr };
    let c_str = unsafe { CStr::from_ptr(s) };
    if let Ok(text) = c_str.to_str() {
        for ch in text.chars() {
            let code = ch as u32;
            ctx.input.push_back(code);
            ctx.record.push(code);
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_input_get(ptr: *mut InputContext) -> c_int {
    match unsafe { ptr.as_mut() }.and_then(|ctx| ctx.input.pop_front()) {
        Some(k) => k as c_int,
        None => -1,
    }
}

#[no_mangle]
pub extern "C" fn rs_input_unget(ptr: *mut InputContext, key: c_uint) {
    if let Some(ctx) = unsafe { ptr.as_mut() } {
        ctx.input.push_front(key as u32);
    }
}

#[no_mangle]
pub extern "C" fn rs_input_avail(ptr: *mut InputContext) -> c_int {
    unsafe { ptr.as_ref() }
        .map(|ctx| (!ctx.input.is_empty()) as c_int)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn rs_redo_feed(ptr: *mut InputContext, key: c_uint) {
    if let Some(ctx) = unsafe { ptr.as_mut() } {
        ctx.redo.push_back(key as u32);
    }
}

#[no_mangle]
pub extern "C" fn rs_redo_feed_str(ptr: *mut InputContext, s: *const c_char) {
    if ptr.is_null() || s.is_null() {
        return;
    }
    let ctx = unsafe { &mut *ptr };
    let c_str = unsafe { CStr::from_ptr(s) };
    if let Ok(text) = c_str.to_str() {
        for ch in text.chars() {
            ctx.redo.push_back(ch as u32);
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_redo_get(ptr: *mut InputContext) -> c_int {
    match unsafe { ptr.as_mut() }.and_then(|ctx| ctx.redo.pop_front()) {
        Some(k) => k as c_int,
        None => -1,
    }
}

fn vec_to_cstr(src: &mut Vec<u32>, buf: *mut c_char, len: usize) -> usize {
    let s: String = src.iter().filter_map(|&c| std::char::from_u32(c)).collect();
    let c_str = match CString::new(s) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let bytes = c_str.as_bytes_with_nul();
    if buf.is_null() || len == 0 {
        return bytes.len();
    }
    if bytes.len() > len {
        return 0;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf as *mut u8, bytes.len());
    }
    src.clear();
    bytes.len()
}

#[no_mangle]
pub extern "C" fn rs_redo_get_all(ptr: *mut InputContext, buf: *mut c_char, len: usize) -> usize {
    if ptr.is_null() {
        return 0;
    }
    let ctx = unsafe { &mut *ptr };
    let mut redo_vec: Vec<u32> = ctx.redo.iter().copied().collect();
    let ret = vec_to_cstr(&mut redo_vec, buf, len);
    if !buf.is_null() && len != 0 && ret != 0 {
        ctx.redo.clear();
    }
    ret
}

#[no_mangle]
pub extern "C" fn rs_record_get(ptr: *mut InputContext, buf: *mut c_char, len: usize) -> usize {
    if ptr.is_null() {
        return 0;
    }
    let ctx = unsafe { &mut *ptr };
    vec_to_cstr(&mut ctx.record, buf, len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn feeds_and_reads_multibyte() {
        let ctx = rs_input_context_new();
        let text = CString::new("aあ").unwrap();
        rs_input_feed_str(ctx, text.as_ptr());
        assert_eq!(rs_input_get(ctx), 'a' as i32);
        assert_eq!(rs_input_get(ctx), 'あ' as i32);
        assert_eq!(rs_input_get(ctx), -1);
        unsafe {
            rs_input_context_free(ctx);
        }
    }

    #[test]
    fn redo_sequence() {
        let ctx = rs_input_context_new();
        rs_redo_feed(ctx, 'x' as u32);
        rs_redo_feed(ctx, 'y' as u32);
        assert_eq!(rs_redo_get(ctx), 'x' as i32);
        assert_eq!(rs_redo_get(ctx), 'y' as i32);
        assert_eq!(rs_redo_get(ctx), -1);
        unsafe {
            rs_input_context_free(ctx);
        }
    }

    #[test]
    fn records_input() {
        let ctx = rs_input_context_new();
        rs_input_feed(ctx, 'a' as u32);
        let needed = rs_record_get(ctx, ptr::null_mut(), 0);
        let mut buf = vec![0i8; needed];
        let got = rs_record_get(ctx, buf.as_mut_ptr(), buf.len());
        assert_eq!(needed, got);
        let s = unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap();
        assert_eq!(s, "a");
        unsafe {
            rs_input_context_free(ctx);
        }
    }

    #[test]
    fn unget_and_avail() {
        let ctx = rs_input_context_new();
        assert_eq!(rs_input_avail(ctx), 0);
        rs_input_feed(ctx, 'a' as u32);
        assert_eq!(rs_input_get(ctx), 'a' as i32);
        assert_eq!(rs_input_avail(ctx), 0);
        rs_input_unget(ctx, 'b' as u32);
        assert_eq!(rs_input_avail(ctx), 1);
        assert_eq!(rs_input_get(ctx), 'b' as i32);
        unsafe {
            rs_input_context_free(ctx);
        }
    }
}
