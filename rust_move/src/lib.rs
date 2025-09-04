use std::os::raw::{c_int, c_long};

// Definition derived from Vim's option handling where `p_sj` represents the
// value of the 'scrolljump' option.  It is declared in Vim's C sources and
// accessed here through FFI.
extern "C" {
    static mut p_sj: c_long;
}

/// Return the scrolljump value to use for a window of height `height`.
///
/// This mirrors the logic from `src/move.c` where a non-negative 'scrolljump'
/// value is used as-is and a negative value is treated as a percentage of the
/// window height.
#[no_mangle]
pub extern "C" fn scrolljump_value(height: c_int) -> c_int {
    unsafe {
        if p_sj >= 0 {
            p_sj as c_int
        } else {
            // When 'scrolljump' is negative it is the percentage of the
            // window height, rounded down.
            height * (-p_sj as c_int) / 100
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Provide a definition for the external `p_sj` option when running tests.
    #[no_mangle]
    static mut p_sj: c_long = 0;

    #[test]
    fn positive_scrolljump_used_directly() {
        unsafe { p_sj = 5; }
        assert_eq!(scrolljump_value(40), 5);
    }

    #[test]
    fn negative_scrolljump_uses_percentage() {
        unsafe { p_sj = -10; }
        // Ten percent of 50 yields 5.
        assert_eq!(scrolljump_value(50), 5);
    }
}
