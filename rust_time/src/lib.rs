#![allow(unexpected_cfgs, clippy::missing_safety_doc, static_mut_refs)]

use libc::{c_char, c_int, strftime, time, time_t, tm};
use std::ptr;

/// Return the current time in seconds.
/// When Vim is built with testing support, a global `time_for_testing`
/// value may be used instead of the system time.
/// # Safety
/// This function may be called from C code.
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

#[cfg(unix)]
unsafe fn vim_localtime(timer: time_t, out: *mut tm) -> *mut tm {
    libc::localtime_r(&timer as *const time_t, out)
}

#[cfg(windows)]
unsafe fn vim_localtime(timer: time_t, out: *mut tm) -> *mut tm {
    if libc::localtime_s(out, &timer) == 0 {
        out
    } else {
        ptr::null_mut()
    }
}

/// Replacement for C's `ctime()` that is safe to call from C code.
/// Returns a pointer to a static buffer containing the formatted time.
/// # Safety
/// The returned pointer becomes invalid on the next call.
#[no_mangle]
pub unsafe extern "C" fn get_ctime(thetime: time_t, add_newline: c_int) -> *const c_char {
    static mut BUF: [c_char; 100] = [0; 100];
    let mut tmres: tm = std::mem::zeroed();
    let tm_ptr = vim_localtime(thetime, &mut tmres as *mut tm);
    if tm_ptr.is_null() {
        let msg: &[u8] = if add_newline != 0 {
            b"(Invalid)\n\0"
        } else {
            b"(Invalid)\0"
        };
        for (i, &b) in msg.iter().enumerate() {
            BUF[i] = b as c_char;
        }
        return BUF.as_ptr();
    }
    let fmt = b"%a %b %d %H:%M:%S %Y\0";
    let len = strftime(
        BUF.as_mut_ptr(),
        BUF.len() - 2,
        fmt.as_ptr() as *const c_char,
        tm_ptr,
    );
    if len == 0 {
        let msg: &[u8] = if add_newline != 0 {
            b"(Invalid)\n\0"
        } else {
            b"(Invalid)\0"
        };
        for (i, &b) in msg.iter().enumerate() {
            BUF[i] = b as c_char;
        }
    } else if add_newline != 0 {
        BUF[len as usize] = b'\n' as c_char;
        BUF[len as usize + 1] = 0;
    }
    BUF.as_ptr()
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
