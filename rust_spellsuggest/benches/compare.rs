use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_spellfile::Trie;
use rust_spellsuggest::suggest;

fn build_trie() -> Trie {
    let mut trie = Trie::new();
    for i in 0..1000 {
        trie.insert(&format!("word{i}"));
    }
    trie
}

fn naive_suggest(trie: &Trie, word: &str, max: usize) -> Vec<String> {
    fn edit_distance_one(a: &str, b: &str) -> bool {
        if a == b {
            return false;
        }
        let la = a.chars().count();
        let lb = b.chars().count();
        if la.abs_diff(lb) > 1 {
            return false;
        }
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let mut i = 0usize;
        let mut j = 0usize;
        let mut diff = 0usize;
        while i < la && j < lb {
            if a_chars[i] == b_chars[j] {
                i += 1;
                j += 1;
            } else {
                diff += 1;
                if diff > 1 {
                    return false;
                }
                if la > lb {
                    i += 1;
                } else if lb > la {
                    j += 1;
                } else {
                    i += 1;
                    j += 1;
                }
            }
        }
        diff += la - i + lb - j;
        diff <= 1
    }

    trie
        .all_words()
        .into_iter()
        .filter(|w| edit_distance_one(word, w))
        .take(max)
        .collect()
}

fn bench_compare(c: &mut Criterion) {
    let trie = build_trie();
    c.bench_function("naive", |b| {
        b.iter(|| naive_suggest(&trie, black_box("word1"), 5))
    });
    c.bench_function("traversal", |b| {
        b.iter(|| suggest(&trie, black_box("word1"), 5))
    });
}

criterion_group!(benches, bench_compare);
criterion_main!(benches);
