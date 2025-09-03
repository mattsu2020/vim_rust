use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

#[no_mangle]
pub extern "C" fn rust_utf_char2len(c: c_int) -> c_int {
    std::char::from_u32(c as u32)
        .map(|ch| {
            let mut buf = [0u8; 4];
            ch.encode_utf8(&mut buf).len() as c_int
        })
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn rust_utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int {
    if buf.is_null() {
        return 0;
    }
    match std::char::from_u32(c as u32) {
        Some(ch) => {
            let mut tmp = [0u8; 4];
            let s = ch.encode_utf8(&mut tmp);
            unsafe {
                ptr::copy_nonoverlapping(s.as_ptr() as *const c_char, buf, s.len());
            }
            s.len() as c_int
        }
        None => 0,
    }
}

fn byte2len(b: u8) -> c_int {
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else if b >> 3 == 0b11110 {
        4
    } else if b >> 2 == 0b111110 {
        5
    } else if b >> 1 == 0b1111110 {
        6
    } else {
        1
    }
}

#[no_mangle]
pub extern "C" fn rust_utf_ptr2len(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }
    unsafe {
        match CStr::from_ptr(p).to_str() {
            Ok(text) => text
                .graphemes(true)
                .next()
                .map(|g| g.len() as c_int)
                .unwrap_or(0),
            Err(_) => 1,
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_utf_byte2len(b: c_int) -> c_int {
    byte2len(b as u8)
}

#[no_mangle]
pub extern "C" fn rust_utf_byte2len_zero(b: c_int) -> c_int {
    let b = b as u8;
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else if b >> 3 == 0b11110 {
        4
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn rust_mb_charlen(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }
    unsafe {
        match CStr::from_ptr(s).to_str() {
            Ok(text) => text.graphemes(true).count() as c_int,
            Err(_) => 0,
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_utf_isupper(c: c_int) -> c_int {
    std::char::from_u32(c as u32)
        .map(|ch| {
            let norm: String = ch.to_string().nfc().collect();
            norm.chars().all(|n| n.is_uppercase()) as c_int
        })
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn rust_utf_islower(c: c_int) -> c_int {
    std::char::from_u32(c as u32)
        .map(|ch| {
            let norm: String = ch.to_string().nfc().collect();
            norm.chars().all(|n| n.is_lowercase()) as c_int
        })
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn rust_utf_toupper(c: c_int) -> c_int {
    std::char::from_u32(c as u32)
        .map(|ch| {
            let norm: String = ch.to_string().nfc().collect();
            norm.chars()
                .flat_map(|n| n.to_uppercase())
                .next()
                .unwrap_or(ch) as c_int
        })
        .unwrap_or(c)
}

#[no_mangle]
pub extern "C" fn rust_utf_tolower(c: c_int) -> c_int {
    std::char::from_u32(c as u32)
        .map(|ch| {
            let norm: String = ch.to_string().nfc().collect();
            norm.chars()
                .flat_map(|n| n.to_lowercase())
                .next()
                .unwrap_or(ch) as c_int
        })
        .unwrap_or(c)
}
