use crate::class::{parse_class, Class};

fn eq_byte(a: u8, b: u8, ic: bool) -> bool {
    if ic {
        a.to_ascii_lowercase() == b.to_ascii_lowercase()
    } else {
        a == b
    }
}

fn match_here(pat: &[u8], text: &[u8], ic: bool) -> Option<usize> {
    if pat.is_empty() {
        return Some(0);
    }
    if pat[0] == b'$' && pat.len() == 1 {
        return if text.is_empty() { Some(0) } else { None };
    }
    if pat[0] == b'[' {
        let (class, len) = parse_class(pat)?;
        if pat.get(len) == Some(&b'*') {
            return match_star_class(&class, &pat[len + 1..], text, ic);
        } else if pat.get(len) == Some(&b'+') {
            if text.is_empty() || !class.matches(text[0], ic) {
                return None;
            }
            return match_star_class(&class, &pat[len + 1..], &text[1..], ic).map(|l| l + 1);
        } else if pat.get(len) == Some(&b'?') {
            if let Some(l) = match_here(&pat[len + 1..], text, ic) {
                return Some(l);
            }
            if !text.is_empty() && class.matches(text[0], ic) {
                return match_here(&pat[len + 1..], &text[1..], ic).map(|l| l + 1);
            }
            return None;
        } else if !text.is_empty() && class.matches(text[0], ic) {
            return match_here(&pat[len..], &text[1..], ic).map(|l| l + 1);
        } else {
            return None;
        }
    }
    let c = pat[0];
    if pat.len() >= 2 {
        match pat[1] {
            b'*' => return match_star_char(c, &pat[2..], text, ic),
            b'+' => {
                if text.is_empty() || (c != b'.' && !eq_byte(c, text[0], ic)) {
                    return None;
                }
                return match_star_char(c, &pat[2..], &text[1..], ic).map(|l| l + 1);
            }
            b'?' => {
                if let Some(l) = match_here(&pat[2..], text, ic) {
                    return Some(l);
                }
                if !text.is_empty() && (c == b'.' || eq_byte(c, text[0], ic)) {
                    return match_here(&pat[2..], &text[1..], ic).map(|l| l + 1);
                }
                return None;
            }
            _ => {}
        }
    }
    if !text.is_empty() && (c == b'.' || eq_byte(c, text[0], ic)) {
        match_here(&pat[1..], &text[1..], ic).map(|l| l + 1)
    } else {
        None
    }
}

fn match_star_char(c: u8, pat: &[u8], text: &[u8], ic: bool) -> Option<usize> {
    let mut i = 0;
    while i < text.len() && (c == b'.' || eq_byte(c, text[i], ic)) {
        i += 1;
    }
    loop {
        if let Some(l) = match_here(pat, &text[i..], ic) {
            return Some(i + l);
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    None
}

fn match_star_class(class: &Class, pat: &[u8], text: &[u8], ic: bool) -> Option<usize> {
    let mut i = 0;
    while i < text.len() && class.matches(text[i], ic) {
        i += 1;
    }
    loop {
        if let Some(l) = match_here(pat, &text[i..], ic) {
            return Some(i + l);
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    None
}

pub fn search(pat: &[u8], text: &[u8], ic: bool) -> Option<(usize, usize)> {
    if pat.first() == Some(&b'^') {
        return match_here(&pat[1..], text, ic).map(|l| (0, l));
    }
    let mut i = 0;
    while i <= text.len() {
        if let Some(l) = match_here(pat, &text[i..], ic) {
            return Some((i, i + l));
        }
        if i == text.len() {
            break;
        }
        i += 1;
    }
    None
}
