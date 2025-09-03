mod ast;
mod bytecode;
mod parser;
mod compiler;
mod executor;

pub use ast::Ast;
pub use bytecode::{Instr, Program, ValueType};
pub use parser::parse_line;
pub use compiler::compile;
pub use executor::execute;

pub fn eval_expr(expr: &str) -> Option<i64> {
    let ast = parse_line(expr)?;
    let prog = compile(&ast);
    if prog.result_type != ValueType::Number {
        return None;
    }
    Some(execute(&prog))
}

pub fn eval_bool_expr(expr: &str) -> Option<bool> {
    let ast = parse_line(expr)?;
    let prog = compile(&ast);
    if prog.result_type != ValueType::Bool {
        return None;
    }
    Some(execute(&prog) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_simple_add() {
        let expr = "1 + 2 + 3";
        assert_eq!(eval_expr(expr), Some(6));
    }

    #[test]
    fn eval_bool_comparison() {
        let expr = "1 < 2";
        assert_eq!(eval_bool_expr(expr), Some(true));
    }
}
