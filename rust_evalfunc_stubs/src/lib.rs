use std::os::raw::c_char;
use std::ffi::c_void;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum vartype_T {
    VAR_UNKNOWN = 0,
    VAR_ANY,
    VAR_VOID,
    VAR_BOOL,
    VAR_SPECIAL,
    VAR_NUMBER,
    VAR_FLOAT,
    VAR_STRING,
    VAR_BLOB,
    VAR_FUNC,
    VAR_PARTIAL,
    VAR_LIST,
    VAR_DICT,
    VAR_JOB,
    VAR_CHANNEL,
    VAR_INSTR,
    VAR_CLASS,
    VAR_OBJECT,
    VAR_TYPEALIAS,
    VAR_TUPLE,
}

#[repr(C)]
pub union vval_union {
    pub v_number: i64,
    pub v_string: *mut c_char,
    pub v_blob: *mut c_void,
    pub v_dict: *mut c_void,
    pub v_list: *mut c_void,
    pub v_partial: *mut c_void,
    pub v_tuple: *mut c_void,
}

#[repr(C)]
pub struct typval_T {
    pub v_type: vartype_T,
    pub v_lock: c_char,
    pub vval: vval_union,
}

unsafe fn ret_zero(rettv: *mut typval_T) {
    if !rettv.is_null() {
        (*rettv).v_type = vartype_T::VAR_NUMBER;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_number = 0;
    }
}

macro_rules! ret_zero_fn {
    ($name:ident) => {
        #[no_mangle]
        pub extern "C" fn $name(_argvars: *mut typval_T, rettv: *mut typval_T) {
            unsafe { ret_zero(rettv) }
        }
    };
}

ret_zero_fn!(f_assert_beeps);
ret_zero_fn!(f_assert_equal);
ret_zero_fn!(f_assert_equalfile);
ret_zero_fn!(f_assert_exception);
ret_zero_fn!(f_assert_fails);
ret_zero_fn!(f_assert_false);
ret_zero_fn!(f_assert_inrange);
ret_zero_fn!(f_assert_match);
ret_zero_fn!(f_assert_nobeep);
ret_zero_fn!(f_assert_notequal);
ret_zero_fn!(f_assert_notmatch);
ret_zero_fn!(f_assert_report);
ret_zero_fn!(f_assert_true);
ret_zero_fn!(f_ch_log);
ret_zero_fn!(f_ch_logfile);
ret_zero_fn!(f_test_alloc_fail);
ret_zero_fn!(f_test_autochdir);
ret_zero_fn!(f_test_feedinput);
ret_zero_fn!(f_test_garbagecollect_now);
ret_zero_fn!(f_test_garbagecollect_soon);
ret_zero_fn!(f_test_getvalue);
ret_zero_fn!(f_test_gui_event);
ret_zero_fn!(f_test_ignore_error);
ret_zero_fn!(f_test_mswin_event);
ret_zero_fn!(f_test_option_not_set);
ret_zero_fn!(f_test_override);
ret_zero_fn!(f_test_refcount);
ret_zero_fn!(f_test_setmouse);
ret_zero_fn!(f_test_settime);

#[no_mangle]
pub extern "C" fn f_test_null_blob(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = vartype_T::VAR_BLOB;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_blob = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_null_dict(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = vartype_T::VAR_DICT;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_dict = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_null_function(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = vartype_T::VAR_FUNC;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_string = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_null_list(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = vartype_T::VAR_LIST;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_list = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_null_partial(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = vartype_T::VAR_PARTIAL;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_partial = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_null_string(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = vartype_T::VAR_STRING;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_string = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_null_tuple(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = vartype_T::VAR_TUPLE;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_tuple = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_unknown(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = vartype_T::VAR_UNKNOWN;
        (*rettv).v_lock = 0;
    }
}

#[no_mangle]
pub extern "C" fn f_test_void(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = vartype_T::VAR_VOID;
        (*rettv).v_lock = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_zero() {
        let mut tv = typval_T {
            v_type: vartype_T::VAR_UNKNOWN,
            v_lock: 0,
            vval: vval_union { v_number: 1 },
        };
        unsafe { f_assert_true(std::ptr::null_mut(), &mut tv); }
        unsafe { assert_eq!(tv.vval.v_number, 0); }
    }
}
