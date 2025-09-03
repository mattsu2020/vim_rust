use crate::ast::Ast;
use crate::bytecode::{Instr, Program, ValueType};

pub fn compile(ast: &Ast) -> Program {
    let mut instrs = Vec::new();
    let result_type = compile_node(ast, &mut instrs);
    Program { instrs, result_type }
}

fn compile_node(ast: &Ast, instrs: &mut Vec<Instr>) -> ValueType {
    match ast {
        Ast::Number(n) => {
            instrs.push(Instr::Const(*n));
            ValueType::Number
        }
        Ast::Add(a, b) => {
            compile_node(a, instrs);
            compile_node(b, instrs);
            instrs.push(Instr::Add);
            ValueType::Number
        }
        Ast::LessThan(a, b) => {
            compile_node(a, instrs);
            compile_node(b, instrs);
            instrs.push(Instr::LessThan);
            ValueType::Bool
        }
        Ast::Echo(expr) => {
            let t = compile_node(expr, instrs);
            instrs.push(Instr::Echo);
            t
        }
    }
}
