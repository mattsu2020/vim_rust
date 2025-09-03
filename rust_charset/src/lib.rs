use std::os::raw::c_uint;

/// Convert the lower 4 bits of a number to its hexadecimal character.
/// Mimics the `nr2hex` helper from `charset.c`.
#[no_mangle]
pub extern "C" fn nr2hex(c: c_uint) -> u8 {
    let n = (c & 0xF) as u8;
    if n <= 9 {
        b'0' + n
    } else {
        b'a' + (n - 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_numbers() {
        assert_eq!(nr2hex(0), b'0');
        assert_eq!(nr2hex(9), b'9');
        assert_eq!(nr2hex(10), b'a');
        assert_eq!(nr2hex(15), b'f');
    }
}
