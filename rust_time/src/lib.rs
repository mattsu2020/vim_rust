use libc::{time, time_t};
use std::ptr;

/// Return the current time in seconds.
/// When Vim is built with testing support, a global `time_for_testing`
/// value may be used instead of the system time.
#[no_mangle]
pub unsafe extern "C" fn vim_time() -> time_t {
    #[cfg(feature = "feat_eval")]
    {
        extern "C" {
            static mut time_for_testing: time_t;
        }
        if time_for_testing != 0 {
            return time_for_testing;
        }
    }
    time(ptr::null_mut())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_time() {
        // Just ensure the function runs and returns a non-zero value.
        let t = unsafe { vim_time() };
        assert!(t > 0);
    }
}
