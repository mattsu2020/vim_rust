use std::fs::File;
use std::io::{self, BufRead, BufReader};

use fst::Set as FstSet;

pub type Set = FstSet<Vec<u8>>;

/// Load a dictionary from a file into an `fst::Set`.
///
/// The file is expected to contain one word per line. Empty lines are ignored.
pub fn load_dict(path: &str) -> io::Result<Set> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut words: Vec<String> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let word = line.trim();
        if !word.is_empty() {
            words.push(word.to_owned());
        }
    }

    words.sort();
    words.dedup();

    FstSet::from_iter(words.into_iter()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn load_dict_basic() {
        let mut path = std::env::temp_dir();
        path.push("dict_test.txt");
        let mut file = File::create(&path).unwrap();
        writeln!(file, "apple").unwrap();
        writeln!(file, "banana").unwrap();
        writeln!(file, "apple").unwrap();

        let set = load_dict(path.to_str().unwrap()).unwrap();
        assert!(set.contains("apple"));
        assert!(set.contains("banana"));
        assert!(!set.contains("orange"));

        fs::remove_file(path).unwrap();
    }
}
