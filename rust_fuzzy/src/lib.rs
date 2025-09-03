/// A simple and safe fuzzy matching algorithm.
///
/// Returns the index of the last matched character in `haystack` if all
/// characters of `needle` appear in order. Matching is case insensitive.
/// The search is performed safely without indexing past buffer boundaries.
pub fn fuzzy_match(needle: &str, haystack: &str) -> Option<usize> {
    let mut iter = haystack.char_indices();
    let mut last = 0usize;
    for ch in needle.chars() {
        match iter.find(|(_, c)| c.eq_ignore_ascii_case(&ch)) {
            Some((idx, _)) => last = idx,
            None => return None,
        }
    }
    Some(last)
}

#[cfg(test)]
mod tests {
    use super::fuzzy_match;

    #[test]
    fn basic_match() {
        assert_eq!(fuzzy_match("fz", "fuzzy"), Some(2));
        assert_eq!(fuzzy_match("abc", "acb"), None);
    }

    #[test]
    fn boundary_safety() {
        let long = "hello";
        assert_eq!(fuzzy_match("hello", long), Some(4));
        assert_eq!(fuzzy_match("hello!", long), None);
    }
}
