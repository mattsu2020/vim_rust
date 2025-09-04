use rust_vim9execute::{execute, Vim9Program};
use rust_vim9instr::Vim9Instr;
use rust_vim9type::Vim9Type;

pub fn parse_line(expr: &str) -> Option<Vec<String>> {
    let tokens: Vec<String> = expr.split_whitespace().map(|s| s.to_string()).collect();
    if tokens.is_empty() {
        None
    } else {
        Some(tokens)
    }
}

pub fn compile(tokens: &[String]) -> Vim9Program {
    let mut instrs = Vec::new();
    let mut iter = tokens.iter();
    if let Some(first) = iter.next() {
        if let Ok(n) = first.parse::<i64>() {
            instrs.push(Vim9Instr::PushNumber(n));
        }
    }
    let mut result_type = Vim9Type::Number;
    while let Some(op) = iter.next() {
        if let Some(num) = iter.next() {
            if let Ok(n) = num.parse::<i64>() {
                instrs.push(Vim9Instr::PushNumber(n));
                match op.as_str() {
                    "+" => instrs.push(Vim9Instr::Add),
                    "<" => {
                        instrs.push(Vim9Instr::CompareLT);
                        result_type = Vim9Type::Bool;
                    }
                    _ => (),
                }
            }
        }
    }
    Vim9Program {
        instrs,
        result_type,
    }
}

pub fn eval_expr(expr: &str) -> Option<i64> {
    let tokens = parse_line(expr)?;
    let prog = compile(&tokens);
    if prog.result_type != Vim9Type::Number {
        return None;
    }
    Some(execute(&prog))
}

pub fn eval_bool_expr(expr: &str) -> Option<bool> {
    let tokens = parse_line(expr)?;
    let prog = compile(&tokens);
    if prog.result_type != Vim9Type::Bool {
        return None;
    }
    Some(execute(&prog) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_add() {
        assert_eq!(eval_expr("1 + 2 + 3"), Some(6));
    }

    #[test]
    fn eval_bool() {
        assert_eq!(eval_bool_expr("1 < 2"), Some(true));
    }
}
