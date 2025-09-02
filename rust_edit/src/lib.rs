//! Editing utilities implemented in Rust.
//!
//! Provides backspace handling that is aware of UTF-8 characters
//! and a simple completion trigger heuristic.

/// Equivalent of Vim's CTRL macro. Shared with C via cbindgen.
#[no_mangle]
pub extern "C" fn ctrl(c: u8) -> u8 {
    c & 0x1f
}

/// Backspace key code derived from `ctrl('h')`.
#[no_mangle]
pub static BACKSPACE: u8 = b'h' & 0x1f;

/// Remove the character before the cursor position, updating the
/// cursor to the new byte index. Handles multi‑byte UTF‑8 characters.
#[no_mangle]
pub fn handle_backspace(text: &mut String, cursor: &mut usize) {
    if *cursor == 0 { return; }
    let mut idx = *cursor;
    // Walk back one UTF‑8 char.
    while idx > 0 {
        idx -= 1;
        if text.is_char_boundary(idx) { break; }
    }
    text.replace_range(idx..*cursor, "");
    *cursor = idx;
}

/// Determine whether a completion should trigger based on the
/// character immediately before the cursor and a slice of trigger
/// characters.
#[no_mangle]
pub fn should_trigger_completion(text: &str, cursor: usize, triggers: &[char]) -> bool {
    if cursor == 0 { return false; }
    match text[..cursor].chars().rev().next() {
        Some(ch) => triggers.contains(&ch),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backspace_ascii() {
        let mut s = String::from("abc");
        let mut cursor = s.len();
        handle_backspace(&mut s, &mut cursor);
        assert_eq!(s, "ab");
        assert_eq!(cursor, 2);
    }

    #[test]
    fn backspace_ime() {
        let mut s = String::from("あい");
        let mut cursor = s.len();
        handle_backspace(&mut s, &mut cursor);
        assert_eq!(s, "あ");
        assert_eq!(cursor, "あ".len());
    }

    #[test]
    fn completion_triggers() {
        let triggers = ['.', '(', 'あ'];
        assert!(should_trigger_completion("foo.", 4, &triggers));
        assert!(should_trigger_completion("いあ", "いあ".len(), &triggers));
        assert!(!should_trigger_completion("bar", 3, &triggers));
    }
}
