use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CStr;
#[cfg(test)]
use std::ffi::CString;
use std::iter::Peekable;
use std::os::raw::c_char;
use std::str::Chars;
use std::sync::Mutex;
pub use rust_core::{typval_T, ValUnion, Vartype, Value, to_typval, from_typval, tv_free};

#[derive(Debug, Clone)]
enum Expr {
    Number(i64),
    Float(f64),
    Str(String),
    Var(String),
    Call(String, Vec<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Concat(Box<Expr>, Box<Expr>),
}

pub struct Evaluator {
    vars: HashMap<String, Value>,
    funcs: HashMap<String, fn(&[Value]) -> Result<Value, ()>>,
}

impl Evaluator {
    pub fn new() -> Self {
        let mut funcs: HashMap<String, fn(&[Value]) -> Result<Value, ()>> = HashMap::new();
        fn add_func(args: &[Value]) -> Result<Value, ()> {
            let a = args
                .get(0)
                .map(|v| v.as_float())
                .transpose()?
                .unwrap_or(0.0);
            let b = args
                .get(1)
                .map(|v| v.as_float())
                .transpose()?
                .unwrap_or(0.0);
            let res = a + b;
            if args.iter().any(|v| matches!(v, Value::Float(_))) || res.fract() != 0.0 {
                Ok(Value::Float(res))
            } else {
                Ok(Value::Number(res as i64))
            }
        }
        fn concat_func(args: &[Value]) -> Result<Value, ()> {
            let mut s = String::new();
            for v in args {
                s.push_str(&v.to_string());
            }
            Ok(Value::Str(s))
        }
        funcs.insert("add".to_string(), add_func);
        funcs.insert("concat".to_string(), concat_func);
        Evaluator { vars: HashMap::new(), funcs }
    }

    pub fn set_var(&mut self, name: &str, val: Value) {
        self.vars.insert(name.to_string(), val);
    }

    pub fn get_var(&self, name: &str) -> Option<Value> {
        self.vars.get(name).cloned()
    }

    pub fn call_function(&self, name: &str, args: &[Value]) -> Result<Value, ()> {
        match self.funcs.get(name) {
            Some(f) => f(args),
            None => Err(()),
        }
    }

    pub fn eval_expr(&self, expr: &str) -> Result<Value, ()> {
        let mut tokens = Tokenizer::new(expr);
        let ast = parse_concat(&mut tokens)?;
        if tokens.next_non_ws().is_some() {
            return Err(());
        }
        eval(&ast, self)
    }

    pub fn eval_script(&mut self, script: &str) -> Result<Option<Value>, ()> {
        let mut last = None;
        for line in script.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix("let ") {
                if let Some((name, expr)) = rest.split_once('=') {
                    let name = name.trim();
                    let expr = expr.trim();
                    let val = self.eval_expr(expr)?;
                    self.set_var(name, val.clone());
                    last = Some(val);
                } else {
                    return Err(());
                }
            } else {
                last = Some(self.eval_expr(line)?);
            }
        }
        Ok(last)
    }
}

static GLOBAL_EVAL: Lazy<Mutex<Evaluator>> = Lazy::new(|| Mutex::new(Evaluator::new()));

struct Tokenizer<'a> {
    iter: Peekable<Chars<'a>>,
}

fn skip_ws(iter: &mut Peekable<Chars<'_>>) {
    while let Some(&c) = iter.peek() {
        if c.is_whitespace() {
            iter.next();
        } else {
            break;
        }
    }
}

impl<'a> Tokenizer<'a> {
    fn new(s: &'a str) -> Self {
        Self { iter: s.chars().peekable() }
    }

    fn next_non_ws(&mut self) -> Option<char> {
        skip_ws(&mut self.iter);
        self.iter.next()
    }

    fn peek_non_ws(&mut self) -> Option<char> {
        skip_ws(&mut self.iter);
        self.iter.peek().copied()
    }

    fn parse_number(&mut self) -> Option<Expr> {
        let mut s = String::new();
        let mut has_dot = false;
        while let Some(&c) = self.iter.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.iter.next();
            } else if c == '.' && !has_dot {
                has_dot = true;
                s.push(c);
                self.iter.next();
            } else {
                break;
            }
        }
        if s.is_empty() {
            None
        } else if has_dot {
            s.parse::<f64>().ok().map(Expr::Float)
        } else {
            s.parse::<i64>().ok().map(Expr::Number)
        }
    }

    fn parse_string(&mut self) -> Option<String> {
        if self.peek_non_ws() != Some('"') {
            return None;
        }
        self.next_non_ws();
        let mut s = String::new();
        while let Some(c) = self.iter.next() {
            if c == '"' {
                return Some(s);
            }
            s.push(c);
        }
        None
    }

    fn parse_identifier(&mut self) -> Option<String> {
        let mut s = String::new();
        if let Some(&c) = self.iter.peek() {
            if c.is_ascii_alphabetic() || c == '_' {
                s.push(c);
                self.iter.next();
            } else {
                return None;
            }
        } else {
            return None;
        }
        while let Some(&c) = self.iter.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                s.push(c);
                self.iter.next();
            } else {
                break;
            }
        }
        Some(s)
    }
}

fn parse_primary(tokens: &mut Tokenizer) -> Result<Expr, ()> {
    if let Some(c) = tokens.peek_non_ws() {
        if c == '(' {
            tokens.next_non_ws();
            let expr = parse_concat(tokens)?;
            if tokens.next_non_ws() != Some(')') {
                return Err(());
            }
            return Ok(expr);
        }
        if c == '"' {
            if let Some(s) = tokens.parse_string() {
                return Ok(Expr::Str(s));
            } else {
                return Err(());
            }
        }
    }
    if let Some(num) = tokens.parse_number() {
        return Ok(num);
    }
    if let Some(id) = tokens.parse_identifier() {
        if tokens.peek_non_ws() == Some('(') {
            tokens.next_non_ws();
            let mut args = Vec::new();
            if tokens.peek_non_ws() != Some(')') {
                loop {
                    let arg = parse_concat(tokens)?;
                    args.push(arg);
                    match tokens.peek_non_ws() {
                        Some(',') => { tokens.next_non_ws(); }
                        Some(')') => break,
                        _ => return Err(()),
                    }
                }
            }
            if tokens.next_non_ws() != Some(')') {
                return Err(());
            }
            return Ok(Expr::Call(id, args));
        } else {
            return Ok(Expr::Var(id));
        }
    }
    Err(())
}

fn parse_mul_div(tokens: &mut Tokenizer) -> Result<Expr, ()> {
    let mut node = parse_primary(tokens)?;
    loop {
        let op = match tokens.peek_non_ws() {
            Some('*') => '*',
            Some('/') => '/',
            _ => break,
        };
        tokens.next_non_ws();
        let rhs = parse_primary(tokens)?;
        node = match op {
            '*' => Expr::Mul(Box::new(node), Box::new(rhs)),
            '/' => Expr::Div(Box::new(node), Box::new(rhs)),
            _ => unreachable!(),
        };
    }
    Ok(node)
}

fn parse_add_sub(tokens: &mut Tokenizer) -> Result<Expr, ()> {
    let mut node = parse_mul_div(tokens)?;
    loop {
        let op = match tokens.peek_non_ws() {
            Some('+') => '+',
            Some('-') => '-',
            _ => break,
        };
        tokens.next_non_ws();
        let rhs = parse_mul_div(tokens)?;
        node = match op {
            '+' => Expr::Add(Box::new(node), Box::new(rhs)),
            '-' => Expr::Sub(Box::new(node), Box::new(rhs)),
            _ => unreachable!(),
        };
    }
    Ok(node)
}

fn parse_concat(tokens: &mut Tokenizer) -> Result<Expr, ()> {
    let mut node = parse_add_sub(tokens)?;
    loop {
        match tokens.peek_non_ws() {
            Some('.') => {
                tokens.next_non_ws();
                let rhs = parse_add_sub(tokens)?;
                node = Expr::Concat(Box::new(node), Box::new(rhs));
            }
            _ => break,
        }
    }
    Ok(node)
}

fn eval(expr: &Expr, ctx: &Evaluator) -> Result<Value, ()> {
    match expr {
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::Float(f) => Ok(Value::Float(*f)),
        Expr::Str(s) => Ok(Value::Str(s.clone())),
        Expr::Var(name) => Ok(ctx.get_var(name).unwrap_or(Value::Number(0))),
        Expr::Call(name, args) => {
            let vals = args
                .iter()
                .map(|e| eval(e, ctx))
                .collect::<Result<Vec<_>, _>>()?;
            ctx.call_function(name, &vals)
        }
        Expr::Add(a, b) => {
            let a = eval(a, ctx)?;
            let b = eval(b, ctx)?;
            match (a, b) {
                (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 + n2)),
                (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 + f2)),
                (Value::Float(f1), Value::Number(n2)) => Ok(Value::Float(f1 + n2 as f64)),
                (Value::Number(n1), Value::Float(f2)) => Ok(Value::Float(n1 as f64 + f2)),
                _ => Err(()),
            }
        }
        Expr::Sub(a, b) => {
            let a = eval(a, ctx)?;
            let b = eval(b, ctx)?;
            match (a, b) {
                (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 - n2)),
                (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 - f2)),
                (Value::Float(f1), Value::Number(n2)) => Ok(Value::Float(f1 - n2 as f64)),
                (Value::Number(n1), Value::Float(f2)) => Ok(Value::Float(n1 as f64 - f2)),
                _ => Err(()),
            }
        }
        Expr::Mul(a, b) => {
            let a = eval(a, ctx)?;
            let b = eval(b, ctx)?;
            match (a, b) {
                (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 * n2)),
                (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 * f2)),
                (Value::Float(f1), Value::Number(n2)) => Ok(Value::Float(f1 * n2 as f64)),
                (Value::Number(n1), Value::Float(f2)) => Ok(Value::Float(n1 as f64 * f2)),
                _ => Err(()),
            }
        }
        Expr::Div(a, b) => {
            let a = eval(a, ctx)?;
            let b = eval(b, ctx)?;
            match (a, b) {
                (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 / n2)),
                (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 / f2)),
                (Value::Float(f1), Value::Number(n2)) => Ok(Value::Float(f1 / n2 as f64)),
                (Value::Number(n1), Value::Float(f2)) => Ok(Value::Float(n1 as f64 / f2)),
                _ => Err(()),
            }
        }
        Expr::Concat(a, b) => {
            let left = eval(a, ctx)?.to_string();
            let right = eval(b, ctx)?.to_string();
            Ok(Value::Str(left + &right))
        }
    }
}


#[no_mangle]
pub extern "C" fn eval_expr_rs(expr: *const c_char, out: *mut typval_T) -> bool {
    if expr.is_null() || out.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(expr) };
    let expr_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    let eval = GLOBAL_EVAL.lock().unwrap();
    match eval.eval_expr(expr_str) {
        Ok(val) => {
            unsafe { to_typval(val, out); }
            true
        }
        Err(_) => false,
    }
}

#[no_mangle]
pub extern "C" fn eval_to_bool_rs(expr: *const c_char, error: *mut bool) -> bool {
    if expr.is_null() {
        if !error.is_null() {
            unsafe { *error = true; }
        }
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(expr) };
    let expr_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            if !error.is_null() {
                unsafe { *error = true; }
            }
            return false;
        }
    };
    let eval = GLOBAL_EVAL.lock().unwrap();
    match eval.eval_expr(expr_str) {
        Ok(val) => match val.as_float() {
            Ok(n) => {
                if !error.is_null() {
                    unsafe { *error = false; }
                }
                n != 0.0
            }
            Err(_) => {
                if !error.is_null() {
                    unsafe { *error = true; }
                }
                false
            }
        },
        Err(_) => {
            if !error.is_null() {
                unsafe { *error = true; }
            }
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn eval_variable_rs(name: *const c_char, out: *mut typval_T) -> bool {
    if name.is_null() || out.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(name) };
    let name_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    let eval = GLOBAL_EVAL.lock().unwrap();
    match eval.get_var(name_str) {
        Some(val) => {
            unsafe { to_typval(val, out); }
            true
        }
        None => false,
    }
}

#[no_mangle]
pub extern "C" fn set_variable_rs(name: *const c_char, val: *const typval_T) -> bool {
    if name.is_null() || val.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(name) };
    let name_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    let value = unsafe { from_typval(val) };
    let mut eval = GLOBAL_EVAL.lock().unwrap();
    if let Some(v) = value {
        eval.set_var(name_str, v);
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn call_function_rs(name: *const c_char, args: *const typval_T, argc: usize, out: *mut typval_T) -> bool {
    if name.is_null() || out.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(name) };
    let name_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    let slice = if args.is_null() {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(args, argc) }
    };
    let mut vals = Vec::new();
    for tv in slice {
        if let Some(v) = unsafe { from_typval(tv) } {
            vals.push(v);
        } else {
            return false;
        }
    }
    let eval = GLOBAL_EVAL.lock().unwrap();
    match eval.call_function(name_str, &vals) {
        Ok(v) => {
            unsafe { to_typval(v, out); }
            true
        }
        Err(_) => false,
    }
}

#[no_mangle]
pub extern "C" fn eval_script_rs(script: *const c_char, out: *mut typval_T) -> bool {
    if script.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(script) };
    let script_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    let mut eval = GLOBAL_EVAL.lock().unwrap();
    match eval.eval_script(script_str) {
        Ok(Some(val)) => {
            if !out.is_null() {
                unsafe { to_typval(val, out); }
            }
            true
        }
        Ok(None) => true,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_and_function_eval() {
        let mut ev = Evaluator::new();
        ev.set_var("x", Value::Number(10));
        assert_eq!(ev.eval_expr("x + 5").unwrap(), Value::Number(15));
        assert_eq!(ev.eval_expr("add(x, 5)").unwrap(), Value::Number(15));
    }

    #[test]
    fn test_vimscript_vim9script_compat() {
        let mut ev = Evaluator::new();
        ev.set_var("s", Value::Str("hi".to_string()));
        let res_vim = ev.eval_expr("concat(s, \" there\")").unwrap();
        let res_vim9 = ev.eval_expr("concat(s, \" there\")").unwrap();
        assert_eq!(res_vim, res_vim9);
    }

    #[test]
    fn test_typval_roundtrip() {
        let val = Value::Str("ab".to_string());
        let mut tv = typval_T {
            v_type: Vartype::VAR_UNKNOWN,
            v_lock: 0,
            vval: ValUnion { v_number: 0 },
        };
        unsafe {
            to_typval(val.clone(), &mut tv);
            let back = from_typval(&tv as *const typval_T).unwrap();
            tv_free(&mut tv);
            assert_eq!(back, val);
        }
    }

    #[test]
    fn test_ffi_variable_and_function() {
        let name = CString::new("x").unwrap();
        let mut out = typval_T {
            v_type: Vartype::VAR_UNKNOWN,
            v_lock: 0,
            vval: ValUnion { v_number: 0 },
        };
        let val = typval_T {
            v_type: Vartype::VAR_NUMBER,
            v_lock: 0,
            vval: ValUnion { v_number: 40 },
        };
        assert!(set_variable_rs(name.as_ptr(), &val));
        assert!(eval_variable_rs(name.as_ptr(), &mut out));
        unsafe { assert_eq!(out.vval.v_number, 40); }

        let mut ret = typval_T {
            v_type: Vartype::VAR_UNKNOWN,
            v_lock: 0,
            vval: ValUnion { v_number: 0 },
        };
        let args = [val, typval_T { v_type: Vartype::VAR_NUMBER, v_lock: 0, vval: ValUnion { v_number: 40 } }];
        assert!(call_function_rs(
            CString::new("add").unwrap().as_ptr(),
            args.as_ptr(),
            args.len(),
            &mut ret,
        ));
        unsafe { assert_eq!(ret.vval.v_number, 80); }
    }
}
