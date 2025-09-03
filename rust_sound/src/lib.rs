use libc::{c_int, c_long, c_char};

#[repr(C)]
#[derive(Clone, Copy)]
pub enum Vartype {
    VAR_UNKNOWN = 0,
    VAR_ANY,
    VAR_VOID,
    VAR_BOOL,
    VAR_SPECIAL,
    VAR_NUMBER,
    VAR_FLOAT,
    VAR_STRING,
}

#[repr(C)]
pub union ValUnion {
    pub v_number: i64,
    pub v_string: *mut c_char,
}

#[repr(C)]
pub struct typval_T {
    pub v_type: Vartype,
    pub v_lock: c_char,
    pub vval: ValUnion,
}

#[repr(C)]
pub struct soundcb_T {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn has_any_sound_callback() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn call_sound_callback(
    _soundcb: *mut soundcb_T,
    _snd_id: c_long,
    _result: c_int,
) {
}

#[no_mangle]
pub extern "C" fn delete_sound_callback(_soundcb: *mut soundcb_T) {
}

#[no_mangle]
pub extern "C" fn has_sound_callback_in_queue() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn invoke_sound_callback() {
}

unsafe fn set_ret_number(rettv: *mut typval_T, num: i64) {
    if rettv.is_null() {
        return;
    }
    (*rettv).v_type = Vartype::VAR_NUMBER;
    (*rettv).v_lock = 0;
    (*rettv).vval.v_number = num;
}

#[no_mangle]
pub unsafe extern "C" fn f_sound_playevent(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
) {
    set_ret_number(rettv, 0);
}

#[no_mangle]
pub unsafe extern "C" fn f_sound_playfile(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
) {
    set_ret_number(rettv, 0);
}

#[no_mangle]
pub unsafe extern "C" fn f_sound_stop(
    _argvars: *mut typval_T,
    _rettv: *mut typval_T,
) {
}

#[no_mangle]
pub unsafe extern "C" fn f_sound_clear(
    _argvars: *mut typval_T,
    _rettv: *mut typval_T,
) {
}

#[no_mangle]
pub extern "C" fn sound_free() {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playevent_returns_zero() {
        unsafe {
            let mut ret = typval_T {
                v_type: Vartype::VAR_UNKNOWN,
                v_lock: 0,
                vval: ValUnion { v_number: -1 },
            };
            f_sound_playevent(std::ptr::null_mut(), &mut ret as *mut _);
            assert_eq!(ret.vval.v_number, 0);
            match ret.v_type {
                Vartype::VAR_NUMBER => {},
                _ => panic!("wrong type"),
            }
        }
    }
}

