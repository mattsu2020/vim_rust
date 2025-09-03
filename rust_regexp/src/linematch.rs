/// Compute the Levenshtein distance between two lines of text.
///
/// This is a very small utility used by a few pieces of the Vim source to
/// measure the similarity of two lines.
pub fn line_match(a: &str, b: &str) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for (i, ca) in a.chars().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur[j + 1] = std::cmp::min(
                std::cmp::min(cur[j] + 1, prev[j + 1] + 1),
                prev[j] + cost,
            );
        }
        prev.clone_from_slice(&cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance() {
        assert_eq!(line_match("kitten", "sitting"), 3);
        assert_eq!(line_match("same", "same"), 0);
    }
}
