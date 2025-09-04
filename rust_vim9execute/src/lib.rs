use rust_vim9instr::Vim9Instr;
use rust_vim9type::Vim9Type;

#[derive(Debug, Clone)]
pub struct Vim9Program {
    pub instrs: Vec<Vim9Instr>,
    pub result_type: Vim9Type,
}

#[derive(Default)]
pub struct Interpreter {
    stack: Vec<i64>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, prog: &Vim9Program) -> i64 {
        self.stack.clear();
        for instr in &prog.instrs {
            match *instr {
                Vim9Instr::PushNumber(n) => self.stack.push(n),
                Vim9Instr::Add => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(a + b);
                    }
                }
                Vim9Instr::Sub => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(a - b);
                    }
                }
                Vim9Instr::CompareLT => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push((a < b) as i64);
                    }
                }
            }
        }
        self.stack.pop().unwrap_or(0)
    }
}

pub fn execute(prog: &Vim9Program) -> i64 {
    Interpreter::new().run(prog)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executes_addition() {
        let prog = Vim9Program {
            instrs: vec![
                Vim9Instr::PushNumber(1),
                Vim9Instr::PushNumber(2),
                Vim9Instr::Add,
            ],
            result_type: Vim9Type::Number,
        };
        assert_eq!(execute(&prog), 3);
    }

    #[test]
    fn executes_comparison() {
        let prog = Vim9Program {
            instrs: vec![
                Vim9Instr::PushNumber(1),
                Vim9Instr::PushNumber(2),
                Vim9Instr::CompareLT,
            ],
            result_type: Vim9Type::Bool,
        };
        assert_eq!(execute(&prog), 1);
    }

    #[test]
    fn executes_subtraction() {
        let prog = Vim9Program {
            instrs: vec![
                Vim9Instr::PushNumber(5),
                Vim9Instr::PushNumber(2),
                Vim9Instr::Sub,
            ],
            result_type: Vim9Type::Number,
        };
        assert_eq!(execute(&prog), 3);
    }
}
