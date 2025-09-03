use crate::bytecode::{Instr, Program};

pub fn execute(prog: &Program) -> i64 {
    let mut stack: Vec<i64> = Vec::new();
    for instr in &prog.instrs {
        match instr {
            Instr::Const(n) => stack.push(*n),
            Instr::Add => {
                let b = stack.pop().unwrap_or(0);
                let a = stack.pop().unwrap_or(0);
                stack.push(a + b);
            }
            Instr::LessThan => {
                let b = stack.pop().unwrap_or(0);
                let a = stack.pop().unwrap_or(0);
                stack.push(if a < b { 1 } else { 0 });
            }
            Instr::Echo => {
                if let Some(v) = stack.last() {
                    println!("{}", v);
                }
            }
        }
    }
    stack.pop().unwrap_or(0)
}
