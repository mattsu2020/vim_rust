#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Class,
}

pub fn type_of(_class: &crate::ast::Class) -> Type {
    Type::Class
}
