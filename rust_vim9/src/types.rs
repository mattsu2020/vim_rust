#[derive(Debug, Clone, PartialEq)]
pub enum Vim9Type {
    Any,
    Number,
    String,
    Bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Vim9Instr {
    Const(i64),
    Add,
}
