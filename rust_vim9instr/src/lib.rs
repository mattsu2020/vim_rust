#[derive(Debug, Clone, PartialEq)]
pub enum Vim9Instr {
    PushNumber(i64),
    Add,
    CompareLT,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_instr() {
        let instr = Vim9Instr::PushNumber(1);
        match instr {
            Vim9Instr::PushNumber(v) => assert_eq!(v, 1),
            _ => panic!("unexpected"),
        }
    }
}
