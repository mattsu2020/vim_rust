use std::ffi::CStr;
use std::os::raw::c_char;

use rust_eval::{typval_T, ValUnion, Vartype};

mod types;
mod parser;
mod compiler;
mod executor;

use parser::parse_line;
use compiler::compile;
use executor::execute;
use types::Vim9Type;

pub fn eval_expr(expr: &str) -> Option<i64> {
    let ast = parse_line(expr)?;
    let prog = compile(&ast);
    if prog.result_type != Vim9Type::Number {
        return None;
    }
    Some(execute(&prog))
}

pub fn eval_bool_expr(expr: &str) -> Option<bool> {
    let ast = parse_line(expr)?;
    let prog = compile(&ast);
    if prog.result_type != Vim9Type::Bool {
        return None;
    }
    Some(execute(&prog) != 0)
}

pub mod cmds {
    use super::*;
    pub fn execute(expr: &str, out: *mut typval_T) -> bool {
        match super::eval_expr(expr) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_eval::eval_expr_rs;

    #[test]
    fn eval_simple_add() {
        let expr = "1 + 2 + 3";
        assert_eq!(eval_expr(expr), Some(6));
    }

    #[test]
    fn echo_command_returns_value() {
        let expr = "echo 4 + 5";
        assert_eq!(eval_expr(expr), Some(9));
    }

    #[test]
    fn compatibility_with_rust_eval() {
        let expr = "1 + 2 + 3";
        let ours = eval_expr(expr).unwrap();
        let c_expr = std::ffi::CString::new(expr).unwrap();
        let mut tv = typval_T { v_type: Vartype::VAR_UNKNOWN, v_lock: 0, vval: ValUnion { v_number: 0 } };
        let ok = eval_expr_rs(c_expr.as_ptr(), &mut tv as *mut typval_T);
        assert!(ok);
        let theirs = unsafe { tv.vval.v_number };
        assert_eq!(ours, theirs);
    }

    #[test]
    fn eval_bool_comparison() {
        let expr = "1 < 2";
        assert_eq!(eval_bool_expr(expr), Some(true));
    }
}
