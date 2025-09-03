use libc::{c_char, c_int, c_uchar};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_skip_to_option_part() {
        let mut s = CString::new(",  test").unwrap();
        let p = unsafe { skip_to_option_part(s.as_ptr() as *mut c_uchar) };
        let res = unsafe { std::ffi::CStr::from_ptr(p as *const c_char) };
        assert_eq!(res.to_str().unwrap(), "test");
    }

    #[test]
    fn test_copy_option_part() {
        let mut opt = CString::new("part1, part2").unwrap();
        let mut p = opt.as_ptr() as *mut c_uchar;
        let mut buf = [0u8; 20];
        let mut option_ptr = p;
        let len = unsafe {
            copy_option_part(
                &mut option_ptr,
                buf.as_mut_ptr(),
                buf.len() as c_int,
                CString::new(",").unwrap().as_ptr(),
            )
        };
        assert_eq!(len, 5);
        assert_eq!(unsafe { std::ffi::CStr::from_ptr(buf.as_ptr() as *const c_char).to_str().unwrap() }, "part1");
    }
}
