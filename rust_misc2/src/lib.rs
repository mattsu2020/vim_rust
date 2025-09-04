use libc::{c_int, c_long, c_uint};

pub const VE_ALL: c_int = 4;
pub const VE_BLOCK: c_int = 5;
pub const VE_INSERT: c_int = 6;
pub const CTRL_V: c_int = 22;
pub const MODE_INSERT: c_int = 0x10;
pub const MAYBE: c_int = 2;

#[no_mangle]
pub static mut virtual_op: c_int = MAYBE;
#[no_mangle]
pub static mut VIsual_active: c_int = 0;
#[no_mangle]
pub static mut VIsual_mode: c_int = 0;
#[no_mangle]
pub static mut State: c_int = 0;

static mut VE_FLAGS: c_uint = 0;

#[no_mangle]
pub extern "C" fn set_ve_flags(flags: c_uint) {
    unsafe {
        VE_FLAGS = flags;
    }
}

#[no_mangle]
pub extern "C" fn get_ve_flags() -> c_uint {
    unsafe { VE_FLAGS }
}

#[no_mangle]
pub extern "C" fn virtual_active() -> c_int {
    unsafe {
        let cur_ve_flags = get_ve_flags() as c_int;
        if virtual_op != MAYBE {
            return virtual_op;
        }
        if cur_ve_flags == VE_ALL
            || ((cur_ve_flags & VE_BLOCK) != 0 && VIsual_active != 0 && VIsual_mode == CTRL_V)
            || ((cur_ve_flags & VE_INSERT) != 0 && (State & MODE_INSERT) != 0)
        {
            1
        } else {
            0
        }
    }
}

// Opaque types used by stub functions.
#[repr(C)]
pub struct pos_T {
    pub lnum: c_long,
    pub col: c_int,
    pub coladd: c_int,
}

#[repr(C)]
pub struct win_T {
    _private: [u8; 0],
}

#[repr(C)]
pub struct buf_T {
    _private: [u8; 0],
}

// Stub implementations for remaining APIs formerly in misc2.c.
#[no_mangle]
pub extern "C" fn getviscol() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn coladvance_force(_wcol: c_long) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn getviscol2(_col: c_long, _coladd: c_long) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn coladvance(_wantcol: c_long) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn getvpos(_pos: *mut pos_T, _wantcol: c_long) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn inc_cursor() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn inc(_lp: *mut pos_T) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn incl(_lp: *mut pos_T) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn dec_cursor() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn dec(_lp: *mut pos_T) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn decl(_lp: *mut pos_T) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn get_cursor_rel_lnum(_wp: *mut win_T, _lnum: c_long) -> c_long {
    0
}

#[no_mangle]
pub extern "C" fn check_pos(_buf: *mut buf_T, _pos: *mut pos_T) {}

#[no_mangle]
pub extern "C" fn check_cursor_lnum() {}

#[no_mangle]
pub extern "C" fn check_cursor_col() {}

#[no_mangle]
pub extern "C" fn check_cursor_col_win(_win: *mut win_T) {}

#[no_mangle]
pub extern "C" fn check_cursor() {}

#[no_mangle]
pub extern "C" fn check_visual_pos() {}

#[no_mangle]
pub extern "C" fn adjust_cursor_col() {}

#[no_mangle]
pub extern "C" fn set_leftcol(_leftcol: c_long) -> c_int {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_active_basic() {
        unsafe {
            virtual_op = MAYBE;
            VIsual_active = 0;
            VIsual_mode = 0;
            State = 0;
            set_ve_flags(0);
        }
        assert_eq!(virtual_active(), 0);
    }

    #[test]
    fn virtual_op_override() {
        unsafe {
            virtual_op = 1;
        }
        assert_eq!(virtual_active(), 1);
        unsafe {
            virtual_op = MAYBE;
        }
    }

    #[test]
    fn virtual_active_flags() {
        unsafe {
            virtual_op = MAYBE;
            VIsual_active = 1;
            VIsual_mode = CTRL_V;
            set_ve_flags(VE_BLOCK as c_uint);
        }
        assert_eq!(virtual_active(), 1);

        unsafe {
            VIsual_active = 0;
            State = MODE_INSERT;
            set_ve_flags(VE_INSERT as c_uint);
        }
        assert_eq!(virtual_active(), 1);
    }
}
