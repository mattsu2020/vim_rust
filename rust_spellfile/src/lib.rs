use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Default)]
pub struct Node {
    pub children: HashMap<char, Node>,
    pub is_word: bool,
}

#[derive(Default)]
pub struct Trie {
    pub root: Node,
}

impl Trie {
    pub fn new() -> Self {
        Self { root: Node::default() }
    }

    pub fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for ch in word.chars() {
            node = node.children.entry(ch).or_default();
        }
        node.is_word = true;
    }

    pub fn contains(&self, word: &str) -> bool {
        let mut node = &self.root;
        for ch in word.chars() {
            match node.children.get(&ch) {
                Some(n) => node = n,
                None => return false,
            }
        }
        node.is_word
    }

    fn collect(node: &Node, prefix: &mut String, out: &mut Vec<String>) {
        if node.is_word {
            out.push(prefix.clone());
        }
        for (ch, child) in &node.children {
            prefix.push(*ch);
            Self::collect(child, prefix, out);
            prefix.pop();
        }
    }

    pub fn all_words(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut prefix = String::new();
        Self::collect(&self.root, &mut prefix, &mut out);
        out
    }
}

pub fn load_dict(path: &str) -> std::io::Result<Trie> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut trie = Trie::new();
    for line in reader.lines() {
        let line = line?;
        let word = line.trim();
        if !word.is_empty() {
            trie.insert(word);
        }
    }
    Ok(trie)
}
