#[derive(Debug, Clone, PartialEq)]
pub enum Vim9Instr {
    PushNumber(i64),
    Add,
    Sub,
    CompareLT,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_instr() {
        let push = Vim9Instr::PushNumber(1);
        match push {
            Vim9Instr::PushNumber(v) => assert_eq!(v, 1),
            _ => panic!("unexpected"),
        }

        let sub = Vim9Instr::Sub;
        assert_eq!(sub, Vim9Instr::Sub);
    }
}
