use libc::{c_int, c_long, c_uint};

pub type colnr_T = c_int;

#[repr(C)]
pub struct pos_T {
    pub lnum: c_long,
    pub col: c_int,
    pub coladd: c_int,
}

extern "C" {
    fn get_ve_flags() -> c_uint;
    static mut virtual_op: c_int;
    static mut VIsual_active: c_int;
    static mut VIsual_mode: c_int;
    static mut State: c_int;

    fn coladvance2(pos: *mut pos_T, addspaces: c_int, finetune: c_int, wcol: colnr_T) -> c_int;
    fn curwin_w_cursor() -> *mut pos_T;
    fn curwin_w_valid() -> *mut c_int;
    fn curwin_w_virtcol() -> *mut colnr_T;
}

const VE_ALL: c_uint = 4;
const VE_BLOCK: c_uint = 5;
const VE_INSERT: c_uint = 6;
const CTRL_V: c_int = 22;
const MODE_INSERT: c_int = 0x10;
const MAYBE: c_int = 2;
const VALID_VIRTCOL: c_int = 0x04;
const MAXCOL: colnr_T = 0x7fffffff;

#[no_mangle]
pub unsafe extern "C" fn virtual_active() -> c_int {
    let cur_ve_flags = get_ve_flags();
    if virtual_op != MAYBE {
        return virtual_op;
    }
    if cur_ve_flags == VE_ALL
        || (cur_ve_flags & VE_BLOCK != 0 && VIsual_active != 0 && VIsual_mode == CTRL_V)
        || (cur_ve_flags & VE_INSERT != 0 && (State & MODE_INSERT) != 0)
    {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn coladvance_force(wcol: colnr_T) -> c_int {
    let rc = coladvance2(curwin_w_cursor(), 1, 0, wcol);
    let w_valid = curwin_w_valid();
    if wcol == MAXCOL {
        *w_valid &= !VALID_VIRTCOL;
    } else {
        *w_valid |= VALID_VIRTCOL;
        *curwin_w_virtcol() = wcol;
    }
    rc
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    static TEST_VE_FLAGS: AtomicU32 = AtomicU32::new(0);

    #[no_mangle]
    unsafe extern "C" fn get_ve_flags() -> c_uint {
        TEST_VE_FLAGS.load(Ordering::Relaxed)
    }
    #[no_mangle]
    static mut virtual_op: c_int = MAYBE;
    #[no_mangle]
    static mut VIsual_active: c_int = 0;
    #[no_mangle]
    static mut VIsual_mode: c_int = 0;
    #[no_mangle]
    static mut State: c_int = 0;

    #[no_mangle]
    unsafe extern "C" fn coladvance2(
        _pos: *mut pos_T,
        _addspaces: c_int,
        _finetune: c_int,
        _wcol: colnr_T,
    ) -> c_int {
        7
    }
    static mut CURSOR: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    static mut W_VALID: c_int = 0;
    static mut W_VIRTCOL: colnr_T = 0;
    #[no_mangle]
    unsafe extern "C" fn curwin_w_cursor() -> *mut pos_T {
        &mut CURSOR
    }
    #[no_mangle]
    unsafe extern "C" fn curwin_w_valid() -> *mut c_int {
        &mut W_VALID
    }
    #[no_mangle]
    unsafe extern "C" fn curwin_w_virtcol() -> *mut colnr_T {
        &mut W_VIRTCOL
    }

    #[test]
    fn test_virtual_active_basic() {
        unsafe {
            virtual_op = 1;
            assert_eq!(virtual_active(), 1);
            virtual_op = MAYBE;
            TEST_VE_FLAGS.store(VE_ALL, Ordering::Relaxed);
            assert_eq!(virtual_active(), 1);
            TEST_VE_FLAGS.store(VE_BLOCK, Ordering::Relaxed);
            VIsual_active = 1;
            VIsual_mode = CTRL_V;
            assert_eq!(virtual_active(), 1);
            VIsual_active = 0;
            VIsual_mode = 0;
            TEST_VE_FLAGS.store(VE_INSERT, Ordering::Relaxed);
            State = MODE_INSERT;
            assert_eq!(virtual_active(), 1);
            State = 0;
            TEST_VE_FLAGS.store(0, Ordering::Relaxed);
            assert_eq!(virtual_active(), 0);
        }
    }

    #[test]
    fn test_coladvance_force_behavior() {
        unsafe {
            W_VALID = 0;
            W_VIRTCOL = 0;
            let rc = coladvance_force(10);
            assert_eq!(rc, 7);
            assert_eq!(W_VALID & VALID_VIRTCOL, VALID_VIRTCOL);
            assert_eq!(W_VIRTCOL, 10);
            let rc2 = coladvance_force(MAXCOL);
            assert_eq!(rc2, 7);
            assert_eq!(W_VALID & VALID_VIRTCOL, 0);
        }
    }
}
