pub fn repeat<T: Clone>(value: T, count: usize) -> Vec<T> {
    (0..count).map(|_| value.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeats_value() {
        let v = repeat(1, 3);
        assert_eq!(v, vec![1, 1, 1]);
    }
}
