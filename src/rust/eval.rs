use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[derive(Debug, Clone)]
enum Expr {
    Number(i64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

fn parse_expr(input: &str) -> Result<Expr, String> {
    let mut chars = Tokenizer::new(input);
    let expr = parse_add_sub(&mut chars)?;
    if let Some(c) = chars.next_non_ws() {
        return Err(format!("unexpected character '{}'", c));
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
}

fn parse_primary(tokens: &mut Tokenizer) -> Result<Expr, String> {
    if let Some(c) = tokens.peek_non_ws() {
        if c == '(' {
            tokens.next_non_ws();
            let expr = parse_add_sub(tokens)?;
            if tokens.next_non_ws() != Some(')') {
                return Err("missing ')'".into());
            }
            return Ok(expr);
        }
    }
    if let Some(num) = tokens.parse_number() {
        Ok(Expr::Number(num))
    } else {
        Err("expected number".into())
    }
}

fn parse_mul_div(tokens: &mut Tokenizer) -> Result<Expr, String> {
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

fn parse_add_sub(tokens: &mut Tokenizer) -> Result<Expr, String> {
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

fn eval(expr: &Expr) -> Result<i64, String> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Add(a, b) => Ok(eval(a)? + eval(b)?),
        Expr::Sub(a, b) => Ok(eval(a)? - eval(b)?),
        Expr::Mul(a, b) => Ok(eval(a)? * eval(b)?),
        Expr::Div(a, b) => {
            let rhs = eval(b)?;
            if rhs == 0 {
                Err("division by zero".into())
            } else {
                Ok(eval(a)? / rhs)
            }
        }
    }
}

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = RefCell::new(None);
}

fn set_error(msg: String) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(CString::new(msg).unwrap());
    });
}

fn clear_error() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}

#[no_mangle]
pub extern "C" fn eval_expr_rs(expr: *const c_char, out: *mut i64) -> bool {
    if expr.is_null() || out.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(expr) };
    let expr_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    match parse_expr(expr_str).and_then(|ast| eval(&ast)) {
        Ok(val) => {
            clear_error();
            unsafe {
                *out = val;
            }
            true
        }
        Err(msg) => {
            set_error(msg);
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn eval_last_error_rs() -> *const c_char {
    LAST_ERROR.with(|e| {
        if let Some(ref s) = *e.borrow() {
            s.as_ptr()
        } else {
            std::ptr::null()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval() {
        let ast = parse_expr("1 + 2 * 3").unwrap();
        assert_eq!(eval(&ast).unwrap(), 7);
    }

    #[test]
    fn test_divide_by_zero() {
        let ast = parse_expr("1/0").unwrap();
        assert!(matches!(eval(&ast), Err(msg) if msg.contains("zero")));
    }

    #[test]
    fn test_parse_error() {
        assert!(parse_expr("1 +").is_err());
    }
}
