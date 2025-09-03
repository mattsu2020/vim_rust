use std::os::raw::c_int;

// Simple state tracking for buffer changes.
static mut DID_WARN: bool = false;
static mut CHANGED: bool = false;
static mut READ_ONLY: bool = true;

/// Equivalent of Vim's change_warning().  Returns 1 when a warning should be
/// displayed and 0 otherwise.  Only warns once while the buffer is marked as
/// readonly and not yet changed.
#[no_mangle]
pub extern "C" fn change_warning(_col: c_int) -> c_int {
    unsafe {
        if DID_WARN || CHANGED || !READ_ONLY {
            return 0;
        }
        DID_WARN = true;
    }
    1
}

/// Mark the buffer as changed.  This mirrors Vim's changed() which notifies
/// the editor that modifications have been made.
#[no_mangle]
pub extern "C" fn changed() {
    unsafe {
        CHANGED = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warns_only_once() {
        unsafe {
            DID_WARN = false;
            CHANGED = false;
            READ_ONLY = true;
        }
        assert_eq!(change_warning(0), 1);
        assert_eq!(change_warning(0), 0);
    }
}

