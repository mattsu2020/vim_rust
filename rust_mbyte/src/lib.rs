use std::os::raw::{c_int, c_uint};
use std::ptr;

#[no_mangle]
pub extern "C" fn utf_char2len(c: c_uint) -> c_int {
    std::char::from_u32(c).map(|ch| ch.len_utf8() as c_int).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn utf_char2bytes(c: c_uint, buf: *mut u8) -> c_int {
    if let Some(ch) = std::char::from_u32(c) {
        let mut tmp = [0u8; 4];
        let s = ch.encode_utf8(&mut tmp);
        unsafe {
            if !buf.is_null() {
                ptr::copy_nonoverlapping(s.as_ptr(), buf, s.len());
            }
        }
        s.len() as c_int
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn utf_ptr2len(p: *const u8) -> c_int {
    if p.is_null() {
        return 0;
    }
    unsafe { utf8_len_of_first_byte(*p) as c_int }
}

#[no_mangle]
pub extern "C" fn utf_ptr2char(p: *const u8, c: *mut c_uint) -> c_int {
    if p.is_null() || c.is_null() {
        return 0;
    }
    unsafe {
        let len = utf8_len_of_first_byte(*p);
        let mut buf = [0u8; 4];
        for i in 0..len {
            buf[i] = *p.add(i);
        }
        if let Ok(s) = std::str::from_utf8(&buf[..len]) {
            if let Some(ch) = s.chars().next() {
                *c = ch as c_uint;
                return len as c_int;
            }
        }
    }
    0
}

fn utf8_len_of_first_byte(b: u8) -> usize {
    match b {
        0x00..=0x7F => 1,
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_multibyte() {
        let ch = '漢';
        assert_eq!(utf_char2len(ch as c_uint), 3);
        let mut buf = [0u8; 4];
        let n = utf_char2bytes(ch as c_uint, buf.as_mut_ptr());
        assert_eq!(n, 3);
        assert_eq!(&buf[..3], ch.to_string().as_bytes());

        let len = utf_ptr2len(buf.as_ptr());
        assert_eq!(len, 3);
        let mut out = 0u32;
        let n2 = utf_ptr2char(buf.as_ptr(), &mut out as *mut c_uint);
        assert_eq!(n2, 3);
        assert_eq!(std::char::from_u32(out).unwrap(), ch);
    }

    #[test]
    fn encode_decode_emoji() {
        let ch = '😀';
        let mut buf = [0u8; 4];
        let n = utf_char2bytes(ch as c_uint, buf.as_mut_ptr());
        assert_eq!(n, 4);
        let mut out = 0u32;
        let n2 = utf_ptr2char(buf.as_ptr(), &mut out as *mut c_uint);
        assert_eq!(n2, 4);
        assert_eq!(std::char::from_u32(out).unwrap(), ch);
    }
}
