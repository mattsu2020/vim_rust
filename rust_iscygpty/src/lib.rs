use std::os::raw::c_int;

/// Stub implementation of the C `is_cygpty` check.
#[no_mangle]
pub extern "C" fn is_cygpty(_fd: c_int) -> c_int {
    0
}

/// Check whether any standard descriptor is a Cygwin pty.
#[no_mangle]
pub extern "C" fn is_cygpty_used() -> c_int {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_zero() {
        assert_eq!(is_cygpty(0), 0);
        assert_eq!(is_cygpty_used(), 0);
    }
}
