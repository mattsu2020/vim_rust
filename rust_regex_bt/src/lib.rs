use regex::Regex;

/// Check whether `text` matches `pattern` using Rust's `regex` crate.
///
/// This provides a simple backtracking matcher replacement for the
/// legacy C implementation.
pub fn is_match(pattern: &str, text: &str) -> bool {
    Regex::new(pattern).map(|re| re.is_match(text)).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_match() {
        assert!(is_match("ab+", "abbb"));
        assert!(!is_match("ab+", "ac"));
    }
}
