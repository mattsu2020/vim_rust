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

#[derive(Debug, Clone)]
enum Expr {
    Number(i64),
    Str(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Concat(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
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

fn parse_expr(input: &str) -> Result<Expr, ()> {
    let mut chars = Tokenizer::new(input);
    let expr = parse_concat(&mut chars)?;
    if chars.next_non_ws().is_some() {
        return Err(());
    }
    Ok(expr)
}

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
        self.next_non_ws(); // skip opening quote
        let mut s = String::new();
        while let Some(c) = self.iter.next() {
            if c == '"' {
                return Some(s);
            }
            s.push(c);
        }
        None
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
        Ok(Expr::Number(num))
    } else {
        Err(())
    }
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

fn eval(expr: &Expr) -> Value {
    match expr {
        Expr::Number(n) => Value::Number(*n),
        Expr::Str(s) => Value::Str(s.clone()),
        Expr::Add(a, b) => {
            let a = eval(a).as_number();
            let b = eval(b).as_number();
            Value::Number(a + b)
        }
        Expr::Sub(a, b) => {
            let a = eval(a).as_number();
            let b = eval(b).as_number();
            Value::Number(a - b)
        }
        Expr::Mul(a, b) => {
            let a = eval(a).as_number();
            let b = eval(b).as_number();
            Value::Number(a * b)
        }
        Expr::Div(a, b) => {
            let a = eval(a).as_number();
            let b = eval(b).as_number();
            Value::Number(a / b)
        }
        Expr::Concat(a, b) => {
            let left = eval(a).to_string();
            let right = eval(b).to_string();
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
    match parse_expr(expr_str) {
        Ok(ast) => {
            let val = eval(&ast);
            unsafe { to_typval(val, out); }
            true
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_number() {
        let ast = parse_expr("1 + 2 * 3").unwrap();
        assert_eq!(eval(&ast), Value::Number(7));
    }

    #[test]
    fn test_eval_string_concat() {
        let ast = parse_expr("\"foo\" . \"bar\"").unwrap();
        assert_eq!(eval(&ast), Value::Str("foobar".to_string()));
    }

    #[test]
    fn test_typval_roundtrip() {
        let ast = parse_expr("\"a\" . \"b\"").unwrap();
        let val = eval(&ast);
        let mut tv = typval_T {
            v_type: Vartype::VAR_UNKNOWN,
            v_lock: 0,
            vval: ValUnion { v_number: 0 },
        };
        unsafe {
            to_typval(val.clone(), &mut tv);
            let back = from_typval(&tv as *const typval_T).unwrap();
            if let Vartype::VAR_STRING = tv.v_type {
                // free allocated string
                let _ = CString::from_raw(tv.vval.v_string);
            }
            assert_eq!(back, val);
        }
    }
}
