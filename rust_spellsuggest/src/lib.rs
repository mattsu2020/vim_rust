use rust_spellfile::Trie;
use std::collections::{HashSet, VecDeque};

/// Suggest words within edit distance one of `word`.
///
/// Traverses the trie while tracking the index into `word` and the number of
/// edits performed. Branches that exceed an edit distance of one are pruned
/// early, avoiding a full traversal of all words in the dictionary.
pub fn suggest(trie: &Trie, word: &str, max: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let chars: Vec<char> = word.chars().collect();
    let mut q: VecDeque<(&rust_spellfile::Node, String, usize, usize)> =
        VecDeque::new();

    q.push_back((&trie.root, String::new(), 0, 0));

    while let Some((node, prefix, idx, edits)) = q.pop_front() {
        if edits > 1 {
            continue;
        }

        if idx == chars.len() {
            if node.is_word && edits == 1 && seen.insert(prefix.clone()) {
                out.push(prefix.clone());
                if out.len() >= max {
                    return out;
                }
            }
            if edits < 1 {
                for (ch, child) in &node.children {
                    let mut new_pref = prefix.clone();
                    new_pref.push(*ch);
                    q.push_back((child, new_pref, idx, edits + 1));
                }
            }
            continue;
        }

        if edits < 1 {
            // deletion
            q.push_back((node, prefix.clone(), idx + 1, edits + 1));
        }

        for (ch, child) in &node.children {
            let mut new_pref = prefix.clone();
            new_pref.push(*ch);
            if *ch == chars[idx] {
                q.push_back((child, new_pref.clone(), idx + 1, edits));
                if edits < 1 {
                    // insertion
                    q.push_back((child, new_pref, idx, edits + 1));
                }
            } else if edits < 1 {
                // substitution
                q.push_back((child, new_pref.clone(), idx + 1, edits + 1));
                // insertion
                q.push_back((child, new_pref, idx, edits + 1));
            }
        }

        if out.len() >= max {
            break;
        }
    }

    out
}

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
