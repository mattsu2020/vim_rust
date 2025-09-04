#![allow(clippy::missing_safety_doc)]
use libc::{c_char, c_int, c_uchar, size_t};

/// Copy a NUL-terminated string into newly allocated memory.
#[no_mangle]
pub unsafe extern "C" fn vim_strsave(string: *const c_uchar) -> *mut c_uchar {
    if string.is_null() {
        return std::ptr::null_mut();
    }
    let len = libc::strlen(string as *const c_char);
    let buf = libc::malloc(len + 1) as *mut c_uchar;
    if buf.is_null() {
        return std::ptr::null_mut();
    }
    std::ptr::copy_nonoverlapping(string, buf, len + 1);
    buf
}

/// Copy up to `len` bytes of `string` into newly allocated memory and terminate with NUL.
#[no_mangle]
pub unsafe extern "C" fn vim_strnsave(string: *const c_uchar, len: size_t) -> *mut c_uchar {
    let buf = libc::malloc(len + 1) as *mut c_uchar;
    if buf.is_null() {
        return std::ptr::null_mut();
    }
    if !string.is_null() {
        std::ptr::copy_nonoverlapping(string, buf, len);
    }
    *buf.add(len) = 0;
    buf
}

/// Skip to next part of an option argument: skip comma and following spaces.
#[no_mangle]
pub unsafe extern "C" fn skip_to_option_part(p: *mut c_uchar) -> *mut c_uchar {
    let mut q = p;
    if *q == b',' as c_uchar {
        q = q.add(1);
    }
    while *q == b' ' as c_uchar {
        q = q.add(1);
    }
    q
}

/// Isolate one part of a string option where parts are separated with
/// `sep_chars`. The part is copied into `buf` which has length `maxlen`.
/// `*option` is advanced to the next part and the length is returned.
#[no_mangle]
pub unsafe extern "C" fn copy_option_part(
    option: *mut *mut c_uchar,
    buf: *mut c_uchar,
    maxlen: c_int,
    sep_chars: *const c_char,
) -> c_int {
    let mut len: c_int = 0;
    let mut p = *option;

    if *p == b'.' as c_uchar {
        *buf.add(len as usize) = *p;
        len += 1;
        p = p.add(1);
    }

    while *p != 0 && libc::strchr(sep_chars, *p as c_int).is_null() {
        if *p == b'\\' as c_uchar {
            let next = *p.add(1);
            if next != 0 && !libc::strchr(sep_chars, next as c_int).is_null() {
                p = p.add(1);
            }
        }
        if len < maxlen - 1 {
            *buf.add(len as usize) = *p;
            len += 1;
        }
        p = p.add(1);
    }
    *buf.add(len as usize) = 0;

    if *p != 0 && *p != b',' as c_uchar {
        p = p.add(1);
    }
    p = skip_to_option_part(p);
    *option = p;
    len
}

/// Vim's own isspace() to handle characters above ASCII 128.
#[no_mangle]
pub extern "C" fn vim_isspace(x: c_int) -> c_int {
    if (9..=13).contains(&x) || x == b' ' as c_int {
        1
    } else {
        0
    }
}
