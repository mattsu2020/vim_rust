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
    Mutex::new(BufferManager {
        bufs: HashMap::new(),
    })
});

/// Locate a buffer by file name and load it if needed.
///
/// Returns a copy of the buffer's contents if the file could be read,
/// otherwise `None`.
pub fn buflist_find_by_name(name: &str) -> Option<Buffer> {
    let mut manager = BUFFER_MANAGER.lock().unwrap();
    if !manager.bufs.contains_key(name) {
        let content = fs::read_to_string(name).ok()?;
        let buf = Buffer {
            lines: content.lines().map(|l| l.to_string()).collect(),
        };
        manager.bufs.insert(name.to_string(), buf);
    }
    manager.bufs.get(name).cloned()
}

/// Return the line `lnum` (1-based) from the buffer with the given `name`.
///
/// The buffer is loaded on demand through `buflist_find_by_name` and the
/// requested line is cloned from the cached contents.  Returns `None` when the
/// buffer or line cannot be found.
pub fn buflist_get_line(name: &str, lnum: usize) -> Option<String> {
    let buf = buflist_find_by_name(name)?;
    buf.lines.get(lnum.checked_sub(1)?).cloned()
}
