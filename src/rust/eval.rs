use std::ffi::CStr;
use std::os::raw::c_char;

#[derive(Debug, Clone)]
enum Expr {
    Number(i64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

fn parse_expr(input: &str) -> Result<Expr, ()> {
    let mut chars = Tokenizer::new(input);
    let expr = parse_add_sub(&mut chars)?;
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
}

fn parse_primary(tokens: &mut Tokenizer) -> Result<Expr, ()> {
    if let Some(c) = tokens.peek_non_ws() {
        if c == '(' {
            tokens.next_non_ws();
            let expr = parse_add_sub(tokens)?;
            if tokens.next_non_ws() != Some(')') {
                return Err(());
            }
            return Ok(expr);
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

fn eval(expr: &Expr) -> i64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::Add(a, b) => eval(a) + eval(b),
        Expr::Sub(a, b) => eval(a) - eval(b),
        Expr::Mul(a, b) => eval(a) * eval(b),
        Expr::Div(a, b) => eval(a) / eval(b),
    }
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
    match parse_expr(expr_str) {
        Ok(ast) => {
            let val = eval(&ast);
            unsafe {
                *out = val;
            }
            true
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval() {
        let ast = parse_expr("1 + 2 * 3").unwrap();
        assert_eq!(eval(&ast), 7);
    }
}
