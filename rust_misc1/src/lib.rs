use libc::{c_int, c_long, c_longlong};

/// For overflow detection, add a digit safely to a long value.
#[no_mangle]
pub unsafe extern "C" fn vim_append_digit_long(value: *mut c_long, digit: c_int) -> c_int {
    if value.is_null() {
        return 0; // FAIL
    }
    let x = *value;
    if x > (c_long::MAX - digit as c_long) / 10 {
        return 0; // FAIL
    }
    *value = x * 10 + digit as c_long;
    1 // OK
}

/// For overflow detection, add a digit safely to an int value.
#[no_mangle]
pub unsafe extern "C" fn vim_append_digit_int(value: *mut c_int, digit: c_int) -> c_int {
    if value.is_null() {
        return 0; // FAIL
    }
    let x = *value;
    if x > (c_int::MAX - digit) / 10 {
        return 0; // FAIL
    }
    *value = x * 10 + digit;
    1 // OK
}

/// Return something that fits into an int.
#[no_mangle]
pub extern "C" fn trim_to_int(x: c_longlong) -> c_int {
    if x > c_int::MAX as c_longlong {
        c_int::MAX
    } else if x < c_int::MIN as c_longlong {
        c_int::MIN
    } else {
        x as c_int
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vim_append_digit_long_ok() {
        let mut v: c_long = 12;
        let r = unsafe { vim_append_digit_long(&mut v as *mut c_long, 3) };
        assert_eq!(r, 1);
        assert_eq!(v, 123);
    }

    #[test]
    fn test_vim_append_digit_long_overflow() {
        let mut v: c_long = c_long::MAX / 10 + 1;
        let r = unsafe { vim_append_digit_long(&mut v as *mut c_long, 5) };
        assert_eq!(r, 0);
    }

    #[test]
    fn test_vim_append_digit_int_ok() {
        let mut v: c_int = 12;
        let r = unsafe { vim_append_digit_int(&mut v as *mut c_int, 3) };
        assert_eq!(r, 1);
        assert_eq!(v, 123);
    }

    #[test]
    fn test_vim_append_digit_int_overflow() {
        let mut v: c_int = c_int::MAX / 10 + 1;
        let r = unsafe { vim_append_digit_int(&mut v as *mut c_int, 5) };
        assert_eq!(r, 0);
    }

    #[test]
    fn test_trim_to_int() {
        assert_eq!(trim_to_int(123), 123);
        assert_eq!(trim_to_int(c_int::MAX as c_longlong + 1), c_int::MAX);
        assert_eq!(trim_to_int(c_int::MIN as c_longlong - 1), c_int::MIN);
    }
}
