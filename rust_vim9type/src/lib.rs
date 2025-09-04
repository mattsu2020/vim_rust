#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vim9Type {
    Number,
    Bool,
}

impl Vim9Type {
    pub fn is_number(self) -> bool {
        matches!(self, Vim9Type::Number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_check() {
        assert!(Vim9Type::Number.is_number());
        assert!(!Vim9Type::Bool.is_number());
    }
}
