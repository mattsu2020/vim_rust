use rust_spellfile::Set;
use strsim::levenshtein;

/// Suggest words from the dictionary that are within an edit distance of 1.
pub fn suggest(dict: &Set, word: &str, max: usize) -> Vec<String> {
    dict.stream()
        .into_strs()
        .unwrap_or_default()
        .into_iter()
        .filter(|candidate| candidate != word && levenshtein(word, candidate) <= 1)
        .take(max)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_spellfile::Set;

    #[test]
    fn suggest_basic() {
        let set = Set::from_iter(["best", "rest", "test", "tests"].into_iter()).unwrap();
        let suggestions = suggest(&set, "test", 2);
        assert_eq!(suggestions, vec!["best", "rest"]);
    }
}
