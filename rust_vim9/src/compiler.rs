use crate::parser::Ast;
use crate::types::{Vim9Instr, Vim9Type};

#[derive(Debug, Clone)]
pub struct Vim9Program {
    pub instrs: Vec<Vim9Instr>,
    pub result_type: Vim9Type,
}

pub fn compile(ast: &Ast) -> Vim9Program {
    let mut instrs = Vec::new();
    let result_type = compile_node(ast, &mut instrs);
    Vim9Program { instrs, result_type }
}

fn compile_node(ast: &Ast, instrs: &mut Vec<Vim9Instr>) -> Vim9Type {
    match ast {
        Ast::Number(n) => {
            instrs.push(Vim9Instr::Const(*n));
            Vim9Type::Number
        }
        Ast::Add(a, b) => {
            compile_node(a, instrs);
            compile_node(b, instrs);
            instrs.push(Vim9Instr::Add);
            Vim9Type::Number
        }
        Ast::LessThan(a, b) => {
            compile_node(a, instrs);
            compile_node(b, instrs);
            instrs.push(Vim9Instr::LessThan);
            Vim9Type::Bool
        }
        Ast::Echo(expr) => {
            let t = compile_node(expr, instrs);
            instrs.push(Vim9Instr::Echo);
            t
        }
    }
}
