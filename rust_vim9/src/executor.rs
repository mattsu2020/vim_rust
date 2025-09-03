use crate::compiler::Vim9Program;
use crate::types::Vim9Instr;

pub fn execute(prog: &Vim9Program) -> i64 {
    let mut stack: Vec<i64> = Vec::new();
    for instr in &prog.instrs {
        match instr {
            Vim9Instr::Const(n) => stack.push(*n),
            Vim9Instr::Add => {
                let b = stack.pop().unwrap_or(0);
                let a = stack.pop().unwrap_or(0);
                stack.push(a + b);
            }
            Vim9Instr::LessThan => {
                let b = stack.pop().unwrap_or(0);
                let a = stack.pop().unwrap_or(0);
                stack.push(if a < b { 1 } else { 0 });
            }
            Vim9Instr::Echo => {
                if let Some(v) = stack.last() {
                    println!("{}", v);
                }
            }
        }
    }
    stack.pop().unwrap_or(0)
}
