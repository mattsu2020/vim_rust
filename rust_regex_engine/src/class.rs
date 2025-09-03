use std::usize;

#[derive(Clone)]
pub enum ClassItem {
    Range(u8, u8),
    PosixDigit,
}

#[derive(Clone)]
pub struct Class {
    negate: bool,
    items: Vec<ClassItem>,
}

impl Class {
    pub fn matches(&self, c: u8, ic: bool) -> bool {
        let mut m = false;
        for item in &self.items {
            match *item {
                ClassItem::Range(a, b) => {
                    if ic {
                        let c1 = c.to_ascii_lowercase();
                        if a.to_ascii_lowercase() <= c1 && c1 <= b.to_ascii_lowercase() {
                            m = true;
                        }
                    } else if a <= c && c <= b {
                        m = true;
                    }
                }
                ClassItem::PosixDigit => {
                    if b'0' <= c && c <= b'9' {
                        m = true;
                    }
                }
            }
            if m {
                break;
            }
        }
        if self.negate { !m } else { m }
    }
}

pub fn parse_class(pat: &[u8]) -> Option<(Class, usize)> {
    if pat.is_empty() || pat[0] != b'[' {
        return None;
    }
    let mut i = 1;
    let negate = if pat.get(i) == Some(&b'^') {
        i += 1;
        true
    } else {
        false
    };
    let mut items = Vec::new();
    while i < pat.len() {
        match pat[i] {
            b']' if i > 1 => {
                return Some((Class { negate, items }, i + 1));
            }
            b'[' if pat.get(i + 1) == Some(&b':') => {
                let mut j = i + 2;
                while j < pat.len() && pat[j] != b':' {
                    j += 1;
                }
                if j + 1 >= pat.len() || pat[j + 1] != b']' {
                    return None;
                }
                let name = &pat[i + 2..j];
                if name == b"digit" {
                    items.push(ClassItem::PosixDigit);
                } else {
                    return None;
                }
                i = j + 2;
            }
            ch => {
                let start = ch;
                i += 1;
                if i + 1 < pat.len() && pat[i] == b'-' && pat[i + 1] != b']' {
                    let end = pat[i + 1];
                    items.push(ClassItem::Range(start, end));
                    i += 2;
                } else {
                    items.push(ClassItem::Range(start, start));
                }
            }
        }
    }
    None
}
