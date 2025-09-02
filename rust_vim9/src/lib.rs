use std::ffi::CStr;
use std::os::raw::c_char;

use rust_eval::{eval_expr_rs, typval_T, ValUnion, Vartype};

pub mod types {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Vim9Type {
        Any,
        Number,
        String,
        Bool,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Vim9Instr {
        Const(i64),
        Add,
    }
}

pub mod compile {
    use super::types::{Vim9Instr, Vim9Type};

    #[derive(Debug, Clone)]
    pub struct Vim9Program {
        pub instrs: Vec<Vim9Instr>,
        pub result_type: Vim9Type,
    }

    pub fn compile_number(expr: &str) -> Vim9Program {
        // very small compiler: only supports numbers and addition
        let parts: Vec<&str> = expr.split('+').collect();
        let mut instrs = Vec::new();
        for p in parts {
            if let Ok(n) = p.trim().parse::<i64>() {
                instrs.push(Vim9Instr::Const(n));
            }
            instrs.push(Vim9Instr::Add);
        }
        Vim9Program { instrs, result_type: Vim9Type::Number }
    }
}

pub mod cmds {
    use super::*;

    pub fn execute(expr: &str, out: *mut typval_T) -> bool {
        let c_string = std::ffi::CString::new(expr).unwrap();
        unsafe { eval_expr_rs(c_string.as_ptr(), out) }
    }
}

#[no_mangle]
pub extern "C" fn vim9_eval_int(expr: *const c_char) -> i64 {
    if expr.is_null() {
        return 0;
    }
    let mut tv = typval_T {
        v_type: Vartype::VAR_UNKNOWN,
        v_lock: 0,
        vval: ValUnion { v_number: 0 },
    };
    let ok = unsafe { eval_expr_rs(expr, &mut tv as *mut typval_T) };
    if !ok {
        return 0;
    }
    unsafe {
        match tv.v_type {
            Vartype::VAR_NUMBER => tv.vval.v_number,
            Vartype::VAR_STRING => {
                if !tv.vval.v_string.is_null() {
                    let _ = std::ffi::CString::from_raw(tv.vval.v_string);
                }
                0
            }
            _ => 0,
        }
    }
}

#[no_mangle]
pub extern "C" fn vim9_exec_rs(expr: *const c_char, out: *mut typval_T) -> bool {
    if expr.is_null() || out.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(expr) };
    match c_str.to_str() {
        Ok(s) => cmds::execute(s, out),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_simple_int() {
        let expr = std::ffi::CString::new("1 + 2 * 3").unwrap();
        let res = unsafe { vim9_eval_int(expr.as_ptr()) };
        assert_eq!(res, 7);
    }
}
