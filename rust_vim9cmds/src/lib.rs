use std::ffi::CStr;
use std::os::raw::c_char;

use rust_eval::{typval_T, ValUnion, Vartype};
use rust_vim9compile::{eval_bool_expr, eval_expr};

pub mod cmds {
    use super::*;
    pub fn execute(expr: &str, out: *mut typval_T) -> bool {
        match eval_expr(expr) {
            Some(n) => unsafe {
                (*out).v_type = Vartype::VAR_NUMBER;
                (*out).vval = ValUnion { v_number: n };
                true
            },
            None => false,
        }
    }
}

#[no_mangle]
pub extern "C" fn vim9_eval_int(expr: *const c_char) -> i64 {
    if expr.is_null() {
        return 0;
    }
    let c_str = unsafe { CStr::from_ptr(expr) };
    match c_str.to_str().ok().and_then(|s| eval_expr(s)) {
        Some(n) => n,
        None => 0,
    }
}

#[no_mangle]
pub extern "C" fn vim9_eval_bool(expr: *const c_char) -> bool {
    if expr.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(expr) };
    match c_str.to_str().ok().and_then(|s| eval_bool_expr(s)) {
        Some(b) => b,
        None => false,
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

#[no_mangle]
pub extern "C" fn vim9_declare_error_rs(name: *const c_char) {
    if name.is_null() {
        return;
    }
    let c_str = unsafe { CStr::from_ptr(name) };
    if let Ok(name) = c_str.to_str() {
        let msg = match name.chars().next().unwrap_or('\0') {
            'g' => format!("cannot declare a global variable {name}"),
            'b' => format!("cannot declare a buffer variable {name}"),
            'w' => format!("cannot declare a window variable {name}"),
            't' => format!("cannot declare a tab variable {name}"),
            'v' => format!("cannot declare a v: variable {name}"),
            '$' => format!("cannot declare an environment variable {name}"),
            '&' => format!("cannot declare an option {name}"),
            '@' => format!("cannot declare a register {name}"),
            _ => return,
        };
        eprintln!("{msg}");
    }
}
