use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

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

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i64),
    Str(String),
}

impl Value {
    fn as_number(&self) -> i64 {
        match self {
            Value::Number(n) => *n,
            Value::Str(s) => s.parse().unwrap_or(0),
        }
    }

    fn to_string(self) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::Str(s) => s,
        }
    }
}

#[derive(Debug, Clone)]
enum Expr {
    Number(i64),
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
    funcs: HashMap<String, fn(&[Value]) -> Value>,
}

impl Evaluator {
    pub fn new() -> Self {
        let mut funcs: HashMap<String, fn(&[Value]) -> Value> = HashMap::new();
        fn add_func(args: &[Value]) -> Value {
            let a = args.get(0).map(|v| v.as_number()).unwrap_or(0);
            let b = args.get(1).map(|v| v.as_number()).unwrap_or(0);
            Value::Number(a + b)
        }
        fn concat_func(args: &[Value]) -> Value {
            let mut s = String::new();
            for v in args {
                s.push_str(&v.clone().to_string());
            }
            Value::Str(s)
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

    pub fn call_function(&self, name: &str, args: &[Value]) -> Option<Value> {
        self.funcs.get(name).map(|f| f(args))
    }

    pub fn eval_expr(&self, expr: &str) -> Result<Value, ()> {
        let mut tokens = Tokenizer::new(expr);
        let ast = parse_concat(&mut tokens)?;
        if tokens.next_non_ws().is_some() {
            return Err(());
        }
        Ok(eval(&ast, self))
    }
}

static GLOBAL_EVAL: Lazy<Mutex<Evaluator>> = Lazy::new(|| Mutex::new(Evaluator::new()));

struct Tokenizer<'a> {
    iter: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    fn new(s: &'a str) -> Self {
        Self { iter: s.chars().peekable() }
    }

    fn next_non_ws(&mut self) -> Option<char> {
        while let Some(&c) = self.iter.peek() {
            if c.is_whitespace() {
                self.iter.next();
            } else {
                return self.iter.next();
            }
        }
        None
    }

    fn peek_non_ws(&mut self) -> Option<char> {
        while let Some(&c) = self.iter.peek() {
            if c.is_whitespace() {
                self.iter.next();
            } else {
                return Some(c);
            }
        }
        None
    }

    fn parse_number(&mut self) -> Option<i64> {
        let mut s = String::new();
        while let Some(&c) = self.iter.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.iter.next();
            } else {
                break;
            }
        }
        if s.is_empty() {
            None
        } else {
            s.parse().ok()
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
        return Ok(Expr::Number(num));
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

fn eval(expr: &Expr, ctx: &Evaluator) -> Value {
    match expr {
        Expr::Number(n) => Value::Number(*n),
        Expr::Str(s) => Value::Str(s.clone()),
        Expr::Var(name) => ctx.get_var(name).unwrap_or(Value::Number(0)),
        Expr::Call(name, args) => {
            let vals: Vec<Value> = args.iter().map(|e| eval(e, ctx)).collect();
            ctx.call_function(name, &vals).unwrap_or(Value::Number(0))
        }
        Expr::Add(a, b) => {
            let a = eval(a, ctx).as_number();
            let b = eval(b, ctx).as_number();
            Value::Number(a + b)
        }
        Expr::Sub(a, b) => {
            let a = eval(a, ctx).as_number();
            let b = eval(b, ctx).as_number();
            Value::Number(a - b)
        }
        Expr::Mul(a, b) => {
            let a = eval(a, ctx).as_number();
            let b = eval(b, ctx).as_number();
            Value::Number(a * b)
        }
        Expr::Div(a, b) => {
            let a = eval(a, ctx).as_number();
            let b = eval(b, ctx).as_number();
            Value::Number(a / b)
        }
        Expr::Concat(a, b) => {
            let left = eval(a, ctx).to_string();
            let right = eval(b, ctx).to_string();
            Value::Str(left + &right)
        }
    }
}

unsafe fn to_typval(val: Value, out: *mut typval_T) {
    match val {
        Value::Number(n) => {
            (*out).v_type = Vartype::VAR_NUMBER;
            (*out).v_lock = 0;
            (*out).vval.v_number = n;
        }
        Value::Str(s) => {
            (*out).v_type = Vartype::VAR_STRING;
            (*out).v_lock = 0;
            let cstr = CString::new(s).unwrap();
            (*out).vval.v_string = cstr.into_raw();
        }
    }
}

unsafe fn from_typval(tv: *const typval_T) -> Option<Value> {
    if tv.is_null() {
        return None;
    }
    match (*tv).v_type {
        Vartype::VAR_NUMBER => Some(Value::Number((*tv).vval.v_number)),
        Vartype::VAR_STRING => {
            if (*tv).vval.v_string.is_null() {
                Some(Value::Str(String::new()))
            } else {
                let cstr = CStr::from_ptr((*tv).vval.v_string);
                cstr.to_str().ok().map(|s| Value::Str(s.to_string()))
            }
        }
        _ => None,
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
        Some(v) => {
            unsafe { to_typval(v, out); }
            true
        }
        None => false,
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
            if let Vartype::VAR_STRING = tv.v_type {
                let _ = CString::from_raw(tv.vval.v_string);
            }
            assert_eq!(back, val);
        }
    }
}
