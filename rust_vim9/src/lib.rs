pub use rust_vim9script::{
    compile, eval_bool_expr, eval_expr, execute, parse_line, Instr as Vim9Instr,
    Program as Vim9Program, ValueType as Vim9Type,
};

/// Execute a Vim9 script consisting of multiple lines.
/// Each line is parsed, compiled and executed independently.
/// Returns a vector with the result of each line.
pub fn execute_script(script: &str) -> Vec<i64> {
    script
        .lines()
        .filter_map(|line| {
            let ast = parse_line(line)?;
            let prog = compile(&ast);
            Some(execute(&prog))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_executes_lines() {
        let script = "1 + 2\n3 + 4\n1 < 2";
        let result = execute_script(script);
        assert_eq!(result, vec![3, 7, 1]);
    }
}
