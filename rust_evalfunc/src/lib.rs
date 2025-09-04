use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;

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
pub union ValUnion {
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
    pub v_type: Vartype,
    pub v_lock: c_char,
    pub vval: ValUnion,
}

#[no_mangle]
pub extern "C" fn f_hostname_rs(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        let mut buf = [0u8; 256];
        if libc::gethostname(buf.as_mut_ptr() as *mut c_char, buf.len()) == 0 {
            buf[buf.len() - 1] = 0; // ensure NUL termination
            let cstr = CStr::from_ptr(buf.as_ptr() as *const c_char);
            let s = CString::new(cstr.to_bytes()).unwrap();
            (*rettv).v_type = Vartype::VAR_STRING;
            (*rettv).v_lock = 0;
            (*rettv).vval.v_string = s.into_raw();
        } else {
            (*rettv).v_type = Vartype::VAR_STRING;
            (*rettv).v_lock = 0;
            (*rettv).vval.v_string = std::ptr::null_mut();
        }
    }
}

#[no_mangle]
pub extern "C" fn f_and_rs(argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        let a = (*argvars).vval.v_number;
        let b = (*argvars.add(1)).vval.v_number;
        (*rettv).v_type = Vartype::VAR_NUMBER;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_number = a & b;
    }
}

unsafe fn ret_zero(rettv: *mut typval_T) {
    if !rettv.is_null() {
        (*rettv).v_type = Vartype::VAR_NUMBER;
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
        (*rettv).v_type = Vartype::VAR_BLOB;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_blob = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_null_dict(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = Vartype::VAR_DICT;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_dict = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_null_function(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = Vartype::VAR_FUNC;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_string = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_null_list(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = Vartype::VAR_LIST;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_list = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_null_partial(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = Vartype::VAR_PARTIAL;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_partial = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_null_string(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = Vartype::VAR_STRING;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_string = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_null_tuple(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = Vartype::VAR_TUPLE;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_tuple = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn f_test_unknown(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = Vartype::VAR_UNKNOWN;
        (*rettv).v_lock = 0;
    }
}

#[no_mangle]
pub extern "C" fn f_test_void(_argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        (*rettv).v_type = Vartype::VAR_VOID;
        (*rettv).v_lock = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn hostname_returns_string() {
        unsafe {
            let mut tv = typval_T {
                v_type: Vartype::VAR_UNKNOWN,
                v_lock: 0,
                vval: ValUnion {
                    v_string: std::ptr::null_mut(),
                },
            };
            f_hostname_rs(std::ptr::null_mut(), &mut tv);
            assert_eq!(tv.v_type as i32, Vartype::VAR_STRING as i32);
            assert!(!tv.vval.v_string.is_null());
            let s = CStr::from_ptr(tv.vval.v_string).to_str().unwrap();
            assert!(!s.is_empty());
            let _ = CString::from_raw(tv.vval.v_string);
        }
    }

    #[test]
    fn and_returns_bitwise_and() {
        unsafe {
            let mut args = [
                typval_T {
                    v_type: Vartype::VAR_NUMBER,
                    v_lock: 0,
                    vval: ValUnion { v_number: 6 },
                },
                typval_T {
                    v_type: Vartype::VAR_NUMBER,
                    v_lock: 0,
                    vval: ValUnion { v_number: 3 },
                },
            ];
            let mut ret = typval_T {
                v_type: Vartype::VAR_UNKNOWN,
                v_lock: 0,
                vval: ValUnion { v_number: 0 },
            };
            f_and_rs(args.as_mut_ptr(), &mut ret);
            assert_eq!(ret.vval.v_number, 2);
        }
    }

    #[test]
    fn stub_returns_zero() {
        let mut tv = typval_T {
            v_type: Vartype::VAR_UNKNOWN,
            v_lock: 0,
            vval: ValUnion { v_number: 1 },
        };
        f_assert_true(std::ptr::null_mut(), &mut tv);
        unsafe { assert_eq!(tv.vval.v_number, 0) };
    }
}
