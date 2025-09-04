use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;

/// Representation of a buffer: a list of lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Buffer {
    pub lines: Vec<String>,
}

struct BufferManager {
    bufs: HashMap<String, Buffer>,
}

static BUFFER_MANAGER: Lazy<Mutex<BufferManager>> = Lazy::new(|| {
    Mutex::new(BufferManager { bufs: HashMap::new() })
});

/// Locate a buffer by file name and load it if needed.
///
/// Returns a copy of the buffer's contents if the file could be read,
/// otherwise `None`.
pub fn buflist_find_by_name(name: &str) -> Option<Buffer> {
    let mut manager = BUFFER_MANAGER.lock().unwrap();
    if !manager.bufs.contains_key(name) {
        let content = fs::read_to_string(name).ok()?;
        let buf = Buffer { lines: content.lines().map(|l| l.to_string()).collect() };
        manager.bufs.insert(name.to_string(), buf);
    }
    manager.bufs.get(name).cloned()
}

#[cfg(test)]
mod tests {
use super::*;
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn loads_and_caches_buffer() {
        let mut path = env::temp_dir();
        path.push("evalbuffer_test.txt");
        let mut file = File::create(&path).unwrap();
        write!(file, "foo\nbar\n").unwrap();

        let path_str = path.to_str().unwrap();
        let buf = buflist_find_by_name(path_str).expect("buffer");
        assert_eq!(buf.lines, vec!["foo".to_string(), "bar".to_string()]);

        // Second call should use cached buffer.
        let cached = buflist_find_by_name(path_str).unwrap();
        assert_eq!(cached.lines, buf.lines);

        fs::remove_file(path).unwrap();
    }
}
