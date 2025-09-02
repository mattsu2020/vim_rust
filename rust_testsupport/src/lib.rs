pub fn vim_iswordc(c: u32) -> bool {
    (c >= b'A' as u32 && c <= b'Z' as u32)
        || (c >= b'a' as u32 && c <= b'z' as u32)
        || (c >= b'0' as u32 && c <= b'9' as u32)
        || c == b'@' as u32
        || c == b'_' as u32
        || (128..=167).contains(&c)
        || (224..=235).contains(&c)
}

pub fn vim_iswordp(bytes: &[u8]) -> bool {
    if let Ok(s) = std::str::from_utf8(bytes) {
        if let Some(ch) = s.chars().next() {
            return vim_iswordc(ch as u32);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isword_funcs_utf8() {
        for c in 0u32..0x10000 {
            if let Some(ch) = std::char::from_u32(c) {
                let mut buf = [0u8; 4];
                let s = ch.encode_utf8(&mut buf);
                assert_eq!(vim_iswordc(c), vim_iswordp(s.as_bytes()), "codepoint {c:#x}");
            }
        }
    }
}
