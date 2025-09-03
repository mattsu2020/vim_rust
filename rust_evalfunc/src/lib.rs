use std::ffi::{CStr, CString};
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
}
