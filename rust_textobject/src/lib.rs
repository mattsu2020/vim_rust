use std::os::raw::{c_int, c_char};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharClass {
    WhiteSpace,
    Punctuation,
    Keyword,
}

pub fn classify(ch: char, bigword: bool) -> CharClass {
    if ch == ' ' || ch == '\t' || ch == '\0' {
        CharClass::WhiteSpace
    } else if bigword {
        CharClass::Keyword
    } else if ch.is_alphanumeric() || ch == '_' {
        CharClass::Keyword
    } else {
        CharClass::Punctuation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

pub fn skip_chars(s: &str, pos: usize, dir: Direction, bigword: bool) -> usize {
    let chars: Vec<char> = s.chars().collect();
    if pos >= chars.len() {
        return pos;
    }
    let class0 = classify(chars[pos], bigword);
    let mut idx = pos;
    match dir {
        Direction::Forward => {
            while idx < chars.len() && classify(chars[idx], bigword) == class0 {
                idx += 1;
            }
        }
        Direction::Backward => {
            while classify(chars[idx], bigword) == class0 {
                if idx == 0 {
                    break;
                }
                idx -= 1;
            }
        }
    }
    idx
}

#[no_mangle]
pub extern "C" fn rust_skip_chars(
    ptr: *const c_char,
    len: usize,
    pos: usize,
    dir: c_int,
    bigword: c_int,
) -> usize {
    if ptr.is_null() {
        return pos;
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
    let s = match std::str::from_utf8(slice) {
        Ok(v) => v,
        Err(_) => return pos,
    };
    let dir = if dir >= 0 {
        Direction::Forward
    } else {
        Direction::Backward
    };
    skip_chars(s, pos, dir, bigword != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify() {
        assert_eq!(classify('a', false), CharClass::Keyword);
        assert_eq!(classify(' ', false), CharClass::WhiteSpace);
        assert_eq!(classify('.', false), CharClass::Punctuation);
        assert_eq!(classify('.', true), CharClass::Keyword);
    }

    #[test]
    fn test_skip_chars_forward() {
        let s = "abc def";
        assert_eq!(skip_chars(s, 0, Direction::Forward, false), 3);
        assert_eq!(skip_chars(s, 3, Direction::Forward, false), 4);
    }

    #[test]
    fn test_skip_chars_backward() {
        let s = "abc def";
        assert_eq!(skip_chars(s, 4, Direction::Backward, false), 3);
    }
}
