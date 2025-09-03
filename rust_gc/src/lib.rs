use std::sync::atomic::{AtomicI32, Ordering};

static COPY_ID: AtomicI32 = AtomicI32::new(0);

/// Return a unique copy identifier.
pub fn get_copy_id() -> i32 {
    COPY_ID.fetch_add(1, Ordering::SeqCst) + 1
}

/// Perform garbage collection.
///
/// This is a placeholder that currently does not free any memory.
pub fn garbage_collect(_testing: bool) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_increase() {
        let id1 = get_copy_id();
        let id2 = get_copy_id();
        assert!(id2 > id1);
    }
}
