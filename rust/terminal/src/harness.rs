use crate::Terminal;
use std::io;

/// Simple test harness for driving a [`Terminal`] instance with byte-oriented
/// input similar to the legacy C harness.
///
/// It supports pushing hexadecimal byte strings into the terminal and
/// retrieving lines from the scrollback buffer for assertions.
pub struct TestHarness {
    term: Terminal,
}

impl TestHarness {
    /// Create a new harness wrapping a [`Terminal`] of the given size.
    pub fn new(width: i32, height: i32) -> io::Result<Self> {
        Ok(Self {
            term: Terminal::new(width, height)?,
        })
    }

    /// Feed a sequence of bytes provided as a hexadecimal string into the
    /// terminal. Whitespace in the string is ignored.
    pub fn push_hex(&mut self, hex: &str) -> io::Result<()> {
        let bytes = hex_to_bytes(hex)?;
        self.term.write_input(&bytes)
    }

    /// Return a line from the terminal scrollback buffer if it exists.
    pub fn scrollback_line(&self, idx: usize) -> Option<String> {
        self.term
            .scrollback()
            .get(idx)
            .map(|s| s.to_string_lossy().into_owned())
    }

    /// Reset the underlying terminal clearing the scrollback buffer.
    pub fn reset(&mut self) {
        self.term.scrollback.clear();
    }
}

fn hex_to_bytes(s: &str) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buf = String::new();
    for ch in s.chars() {
        if ch.is_whitespace() {
            continue;
        }
        buf.push(ch);
        if buf.len() == 2 {
            let byte = u8::from_str_radix(&buf, 16)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            bytes.push(byte);
            buf.clear();
        }
    }
    if !buf.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "hex string has odd length",
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_to_bytes_parses() {
        let bytes = hex_to_bytes("68656c6c6f").unwrap();
        assert_eq!(bytes, b"hello");
    }
}

