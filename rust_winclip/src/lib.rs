use libc::{c_char, c_int, c_long, c_ulong, c_void};
use std::ptr;
use std::slice;

use rust_clipboard::{get_string, set_string};

const MCHAR: c_int = 0;

#[repr(C)]
pub struct ClipboardT {
    _private: [u8; 0],
}

extern "C" {
    fn clip_init(use_cb: c_int);
    fn clip_yank_selection(regtype: c_int, data: *const u8, len: c_long, cbd: *mut ClipboardT);
    fn clip_get_selection(cbd: *mut ClipboardT);
    fn clip_convert_selection(strp: *mut *mut u8, lenp: *mut c_ulong, cbd: *mut ClipboardT) -> c_int;
    fn vim_free(ptr: *mut c_void);
}

#[no_mangle]
pub extern "C" fn win_clip_init() {
    unsafe { clip_init(1) };
}

#[no_mangle]
pub extern "C" fn clip_mch_own_selection(_cbd: *mut ClipboardT) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn clip_mch_lose_selection(_cbd: *mut ClipboardT) {}

#[no_mangle]
pub extern "C" fn clip_mch_request_selection(cbd: *mut ClipboardT) {
    if let Some(text) = get_string() {
        unsafe {
            clip_yank_selection(MCHAR, text.as_ptr(), text.len() as c_long, cbd);
        }
    }
}

#[no_mangle]
pub extern "C" fn clip_mch_set_selection(cbd: *mut ClipboardT) {
    unsafe {
        clip_get_selection(cbd);
        let mut str_ptr: *mut u8 = ptr::null_mut();
        let mut len: c_ulong = 0;
        if clip_convert_selection(&mut str_ptr, &mut len, cbd) >= 0 && !str_ptr.is_null() {
            let slice = slice::from_raw_parts(str_ptr, len as usize);
            if let Ok(s) = std::str::from_utf8(slice) {
                let _ = set_string(s);
            }
        }
        if !str_ptr.is_null() {
            vim_free(str_ptr as *mut c_void);
        }
    }
}

#[no_mangle]
pub extern "C" fn utf8_to_utf16(instr: *const u8, inlen: c_int, outstr: *mut u16, unconvlenp: *mut c_int) -> c_int {
    let slice = unsafe { slice::from_raw_parts(instr, inlen as usize) };
    let s = match std::str::from_utf8(slice) {
        Ok(v) => v,
        Err(_) => {
            if !unconvlenp.is_null() {
                unsafe { *unconvlenp = inlen };
            }
            return 0;
        }
    };
    if outstr.is_null() {
        return s.encode_utf16().count() as c_int;
    }
    let mut count = 0;
    for (i, c) in s.encode_utf16().enumerate() {
        unsafe { *outstr.add(i) = c; }
        count += 1;
    }
    count as c_int
}

#[no_mangle]
pub extern "C" fn utf16_to_utf8(instr: *const u16, inlen: c_int, outstr: *mut u8) -> c_int {
    let slice = unsafe { slice::from_raw_parts(instr, inlen as usize) };
    let s = String::from_utf16_lossy(slice);
    if outstr.is_null() {
        return s.len() as c_int;
    }
    unsafe {
        ptr::copy_nonoverlapping(s.as_ptr(), outstr, s.len());
    }
    s.len() as c_int
}

#[no_mangle]
pub extern "C" fn MultiByteToWideChar_alloc(_cp: u32, _flags: u32, input: *const c_char, inlen: c_int, out: *mut *mut u16, outlen: *mut c_int) {
    if input.is_null() || out.is_null() || outlen.is_null() {
        return;
    }
    let slice = unsafe { slice::from_raw_parts(input as *const u8, inlen as usize) };
    let s = std::str::from_utf8(slice).unwrap_or("");
    let mut data: Vec<u16> = s.encode_utf16().collect();
    data.push(0);
    unsafe {
        *outlen = (data.len() - 1) as c_int;
        let size = data.len() * std::mem::size_of::<u16>();
        let ptr = libc::malloc(size) as *mut u16;
        if ptr.is_null() {
            *out = ptr::null_mut();
            return;
        }
        ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
        *out = ptr;
    }
}

#[no_mangle]
pub extern "C" fn WideCharToMultiByte_alloc(_cp: u32, _flags: u32, input: *const u16, inlen: c_int, out: *mut *mut c_char, outlen: *mut c_int, _def: *const c_char, _useddef: *mut c_int) {
    if input.is_null() || out.is_null() || outlen.is_null() {
        return;
    }
    let slice = unsafe { slice::from_raw_parts(input, inlen as usize) };
    let s = String::from_utf16_lossy(slice);
    unsafe {
        *outlen = s.len() as c_int;
        let ptr = libc::malloc(s.len() + 1) as *mut u8;
        if ptr.is_null() {
            *out = ptr::null_mut();
            return;
        }
        ptr::copy_nonoverlapping(s.as_ptr(), ptr, s.len());
        *ptr.add(s.len()) = 0;
        *out = ptr as *mut c_char;
    }
}

#[no_mangle]
pub extern "C" fn enc_to_utf16(str: *mut u8, lenp: *mut c_int) -> *mut u16 {
    if str.is_null() {
        return ptr::null_mut();
    }
    let len = if lenp.is_null() {
        unsafe { libc::strlen(str as *const c_char) as usize }
    } else {
        unsafe { *lenp as usize }
    };
    let slice = unsafe { slice::from_raw_parts(str, len) };
    let s = std::str::from_utf8(slice).unwrap_or("");
    let mut data: Vec<u16> = s.encode_utf16().collect();
    if !lenp.is_null() {
        unsafe { *lenp = data.len() as c_int; }
    }
    data.push(0);
    let ptr_out = unsafe { libc::malloc(data.len() * 2) as *mut u16 };
    if ptr_out.is_null() {
        return ptr::null_mut();
    }
    unsafe { ptr::copy_nonoverlapping(data.as_ptr(), ptr_out, data.len()); }
    ptr_out
}

#[no_mangle]
pub extern "C" fn utf16_to_enc(str: *mut u16, lenp: *mut c_int) -> *mut u8 {
    if str.is_null() {
        return ptr::null_mut();
    }
    let len = if lenp.is_null() {
        let mut p = str;
        let mut n = 0;
        unsafe {
            while *p != 0 {
                n += 1;
                p = p.add(1);
            }
        }
        n
    } else {
        unsafe { *lenp as usize }
    };
    let slice = unsafe { slice::from_raw_parts(str, len) };
    let s = String::from_utf16_lossy(slice);
    if !lenp.is_null() {
        unsafe { *lenp = s.len() as c_int; }
    }
    let mut bytes = s.into_bytes();
    bytes.push(0);
    let ptr_out = unsafe { libc::malloc(bytes.len()) as *mut u8 };
    if ptr_out.is_null() {
        return ptr::null_mut();
    }
    unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), ptr_out, bytes.len()); }
    ptr_out
}

#[no_mangle]
pub extern "C" fn acp_to_enc(str: *mut u8, str_size: c_int, out: *mut *mut u8, outlen: *mut c_int) {
    if out.is_null() || outlen.is_null() {
        return;
    }
    unsafe {
        *outlen = str_size;
        let ptr_out = libc::malloc((str_size as usize + 1) as usize) as *mut u8;
        if ptr_out.is_null() {
            *out = ptr::null_mut();
            return;
        }
        ptr::copy_nonoverlapping(str, ptr_out, str_size as usize);
        *ptr_out.add(str_size as usize) = 0;
        *out = ptr_out;
    }
}

#[no_mangle]
pub extern "C" fn enc_to_acp(str: *mut u8, str_size: c_int, out: *mut *mut u8, outlen: *mut c_int) {
    acp_to_enc(str, str_size, out, outlen);
}
