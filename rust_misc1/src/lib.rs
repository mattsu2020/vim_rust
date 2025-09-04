use libc::{c_int, c_long, c_longlong};

const OK: c_int = 1;
const FAIL: c_int = 0;

#[no_mangle]
pub extern "C" fn vim_append_digit_long(value: *mut c_long, digit: c_int) -> c_int {
    if value.is_null() {
        return FAIL;
    }
    unsafe {
        let x = *value;
        if x > ((c_long::MAX - digit as c_long) / 10) {
            return FAIL;
        }
        *value = x.wrapping_mul(10) + digit as c_long;
    }
    OK
}

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
