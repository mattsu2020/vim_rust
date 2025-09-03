#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Any,
    Number,
    String,
    Bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    Const(i64),
    Add,
    LessThan,
    Echo,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub instrs: Vec<Instr>,
    pub result_type: ValueType,
}
