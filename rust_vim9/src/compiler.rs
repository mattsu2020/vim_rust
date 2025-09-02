use crate::parser::Ast;
use crate::types::{Vim9Instr, Vim9Type};

#[derive(Debug, Clone)]
pub struct Vim9Program {
    pub instrs: Vec<Vim9Instr>,
    pub result_type: Vim9Type,
}

pub fn compile(ast: &Ast) -> Vim9Program {
    let mut instrs = Vec::new();
    compile_node(ast, &mut instrs);
    Vim9Program { instrs, result_type: Vim9Type::Number }
}

fn compile_node(ast: &Ast, instrs: &mut Vec<Vim9Instr>) {
    match ast {
        Ast::Number(n) => instrs.push(Vim9Instr::Const(*n)),
        Ast::Add(a, b) => {
            compile_node(a, instrs);
            compile_node(b, instrs);
            instrs.push(Vim9Instr::Add);
        }
        Ast::Echo(expr) => {
            compile_node(expr, instrs);
            instrs.push(Vim9Instr::Echo);
        }
    }
}
