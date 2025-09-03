/// Very small and naive fuzzy matcher.
///
/// The function checks whether all characters of `needle` appear in `haystack`
/// in the same order.  Matching is case insensitive.  When successful the
/// index of the last matched character is returned.
pub fn fuzzy_match(needle: &str, haystack: &str) -> Option<usize> {
    let mut iter = haystack.chars().enumerate();
    let mut last = 0usize;
    for ch in needle.chars() {
        match iter.find(|(_, c)| c.eq_ignore_ascii_case(&ch)) {
            Some((idx, _)) => {
                last = idx;
            }
            None => return None,
        }
    }
    Some(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_fuzzy() {
        assert_eq!(fuzzy_match("fz", "fuzzy"), Some(2));
        assert_eq!(fuzzy_match("abc", "acb"), None);
    }
}
