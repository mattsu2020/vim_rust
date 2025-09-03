#![cfg(windows)]

use std::sync::atomic::{AtomicUsize, Ordering};

static G_HINST: AtomicUsize = AtomicUsize::new(0);

/// Store the instance handle of the executable or DLL.
#[no_mangle]
pub extern "C" fn SaveInst(h_inst: usize) {
    G_HINST.store(h_inst, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_instance() {
        SaveInst(1234);
        assert_eq!(G_HINST.load(Ordering::Relaxed), 1234);
    }
}
