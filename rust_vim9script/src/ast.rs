#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
    Number(i64),
    Add(Box<Ast>, Box<Ast>),
    LessThan(Box<Ast>, Box<Ast>),
    Echo(Box<Ast>),
}
