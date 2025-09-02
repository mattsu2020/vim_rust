use std::os::raw::{c_int, c_uint, c_uchar};

fn is_xdigit(c: u8) -> bool {
    matches!(c, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')
}

#[unsafe(no_mangle)]
pub extern "C" fn nr2hex(c: c_uint) -> c_uint {
    let c = c & 0xf;
    if c <= 9 {
        c + b'0' as c_uint
    } else {
        c - 10 + b'a' as c_uint
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn hex2nr(c: c_int) -> c_int {
    if (b'a' as c_int..=b'f' as c_int).contains(&c) {
        c - b'a' as c_int + 10
    } else if (b'A' as c_int..=b'F' as c_int).contains(&c) {
        c - b'A' as c_int + 10
    } else {
        c - b'0' as c_int
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn hexhex2nr(p: *const c_uchar) -> c_int {
    if p.is_null() {
        return -1;
    }
    unsafe {
        let b0 = *p;
        let b1 = *p.add(1);
        if !is_xdigit(b0) || !is_xdigit(b1) {
            return -1;
        }
        (hex2nr(b0 as c_int) << 4) + hex2nr(b1 as c_int)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nr2hex() {
        assert_eq!(nr2hex(0), '0' as c_uint);
        assert_eq!(nr2hex(10), 'a' as c_uint);
        assert_eq!(nr2hex(15), 'f' as c_uint);
    }

    #[test]
    fn test_hex2nr() {
        assert_eq!(hex2nr('0' as c_int), 0);
        assert_eq!(hex2nr('9' as c_int), 9);
        assert_eq!(hex2nr('a' as c_int), 10);
        assert_eq!(hex2nr('F' as c_int), 15);
    }

    #[test]
    fn test_hexhex2nr() {
        let bytes = b"4f";
        let res = hexhex2nr(bytes.as_ptr());
        assert_eq!(res, 0x4f);
        let bad = b"g0";
        let res = hexhex2nr(bad.as_ptr());
        assert_eq!(res, -1);
    }
}
