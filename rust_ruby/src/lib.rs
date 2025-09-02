use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::OnceLock;
use magnus::{eval, RString, Value};

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

static RUBY: OnceLock<magnus::embed::Cleanup> = OnceLock::new();

fn init_ruby() {
    RUBY.get_or_init(|| unsafe { magnus::embed::init() });
}

unsafe fn to_typval(val: Value, out: *mut typval_T) {
    if let Ok(n) = val.try_convert::<i64>() {
        (*out).v_type = Vartype::VAR_NUMBER;
        (*out).v_lock = 0;
        (*out).vval.v_number = n;
    } else if let Ok(s) = val.try_convert::<RString>() {
        (*out).v_type = Vartype::VAR_STRING;
        (*out).v_lock = 0;
        let cstr = CString::new(s.to_string().unwrap()).unwrap();
        (*out).vval.v_string = cstr.into_raw();
    } else {
        (*out).v_type = Vartype::VAR_NUMBER;
        (*out).v_lock = 0;
        (*out).vval.v_number = 0;
    }
}

#[no_mangle]
pub extern "C" fn do_rubyeval(expr: *const c_char, out: *mut typval_T) {
    if expr.is_null() || out.is_null() {
        return;
    }
    init_ruby();
    let c_str = unsafe { CStr::from_ptr(expr) };
    let code = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe {
                (*out).v_type = Vartype::VAR_NUMBER;
                (*out).v_lock = 0;
                (*out).vval.v_number = 0;
            }
            return;
        }
    };
    match eval::<Value>(code) {
        Ok(val) => unsafe { to_typval(val, out) },
        Err(_) => unsafe {
            (*out).v_type = Vartype::VAR_NUMBER;
            (*out).v_lock = 0;
            (*out).vval.v_number = 0;
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_ruby_eval_number() {
        let mut tv = typval_T { v_type: Vartype::VAR_UNKNOWN, v_lock: 0, vval: ValUnion { v_number: 0 } };
        let code = CString::new("1 + 2").unwrap();
        do_rubyeval(code.as_ptr(), &mut tv as *mut typval_T);
        assert_eq!(tv.v_type as i32, Vartype::VAR_NUMBER as i32);
        unsafe { assert_eq!(tv.vval.v_number, 3); }
    }

    #[test]
    #[ignore]
    fn test_ruby_eval_string() {
        let mut tv = typval_T { v_type: Vartype::VAR_UNKNOWN, v_lock: 0, vval: ValUnion { v_number: 0 } };
        let code = CString::new("'foo' + 'bar'").unwrap();
        do_rubyeval(code.as_ptr(), &mut tv as *mut typval_T);
        assert_eq!(tv.v_type as i32, Vartype::VAR_STRING as i32);
        unsafe {
            let s = CStr::from_ptr(tv.vval.v_string).to_str().unwrap();
            assert_eq!(s, "foobar");
            let _ = CString::from_raw(tv.vval.v_string);
        }
    }
}
