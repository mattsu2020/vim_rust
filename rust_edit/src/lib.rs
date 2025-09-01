use libc::c_int;

static mut TEXT: Vec<char> = Vec::new();
static mut CURSOR: usize = 0;

#[no_mangle]
pub extern "C" fn rs_edit_reset() {
    unsafe {
        TEXT.clear();
        CURSOR = 0;
    }
}

#[no_mangle]
pub extern "C" fn rs_ins_char(c: c_int) -> c_int {
    let ch = std::char::from_u32(c as u32).unwrap_or('\0');
    unsafe {
        if CURSOR > TEXT.len() {
            CURSOR = TEXT.len();
        }
        TEXT.insert(CURSOR, ch);
        CURSOR += 1;
    }
    1
}

#[no_mangle]
pub extern "C" fn rs_del_char(_fixpos: c_int) -> c_int {
    unsafe {
        if CURSOR < TEXT.len() {
            TEXT.remove(CURSOR);
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_backspace_char() -> c_int {
    unsafe {
        if CURSOR > 0 {
            CURSOR -= 1;
            TEXT.remove(CURSOR);
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_edit_flow() {
        rs_edit_reset();
        assert_eq!(rs_ins_char('a' as c_int), 1);
        assert_eq!(rs_ins_char('b' as c_int), 1);
        assert_eq!(rs_backspace_char(), 1); // remove 'b'
        unsafe { CURSOR = 0; }
        assert_eq!(rs_del_char(0), 1); // delete 'a'
        unsafe {
            assert_eq!(TEXT.len(), 0);
            assert_eq!(CURSOR, 0);
        }
    }
}
