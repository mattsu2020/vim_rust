use libc::c_char;
use rust_evalvars::rs_win_getid;

#[allow(non_camel_case_types)]
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
}

#[repr(C)]
pub struct typval_T {
    pub v_type: Vartype,
    pub v_lock: c_char,
    pub vval: ValUnion,
}

#[no_mangle]
pub extern "C" fn f_win_getid(argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        let winnr = if (*argvars).v_type as i32 == Vartype::VAR_UNKNOWN as i32 {
            0
        } else {
            (*argvars).vval.v_number as i32
        };
        let id = rs_win_getid(winnr);
        (*rettv).v_type = Vartype::VAR_NUMBER;
        (*rettv).v_lock = 0;
        (*rettv).vval.v_number = id as i64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_evalvars::{rs_win_create, rs_win_getid};

    #[test]
    fn returns_window_id_for_number() {
        unsafe {
            rs_win_create();
            rs_win_create();
            let mut args = [typval_T {
                v_type: Vartype::VAR_NUMBER,
                v_lock: 0,
                vval: ValUnion { v_number: 2 },
            }];
            let mut ret = typval_T {
                v_type: Vartype::VAR_UNKNOWN,
                v_lock: 0,
                vval: ValUnion { v_number: 0 },
            };
            f_win_getid(args.as_mut_ptr(), &mut ret);
            assert_eq!(ret.vval.v_number, rs_win_getid(2) as i64);
        }
    }

    #[test]
    fn defaults_to_current_window() {
        unsafe {
            rs_win_create();
            let mut args = [typval_T {
                v_type: Vartype::VAR_UNKNOWN,
                v_lock: 0,
                vval: ValUnion { v_number: 0 },
            }];
            let mut ret = typval_T {
                v_type: Vartype::VAR_UNKNOWN,
                v_lock: 0,
                vval: ValUnion { v_number: 0 },
            };
            f_win_getid(args.as_mut_ptr(), &mut ret);
            assert_eq!(ret.vval.v_number, rs_win_getid(0) as i64);
        }
    }
}
