use libc::{c_char, c_int, c_uchar};
use std::ffi::CString;
use std::ptr;
use std::sync::Mutex;

// Constants matching Vim's option.h definitions.
const EOL_UNIX: c_int = 0;
const EOL_DOS: c_int = 1;
const EOL_MAC: c_int = 2;
const FORCE_BIN: c_int = 1;

// Cached user name string protected by a mutex for safe access.
static USERNAME: Mutex<Option<CString>> = Mutex::new(None);

/// Get the current user name, caching the result.
///
/// `buf` must point to a writable buffer of length `len` where the
/// NUL-terminated name will be written. Returns 1 on success and 0 on failure.
#[no_mangle]
/// # Safety
/// `buf` must be valid for writes of `len` bytes.
pub unsafe extern "C" fn get_user_name(buf: *mut c_uchar, len: c_int) -> c_int {
    if buf.is_null() || len <= 0 {
        return 0; // FAIL
    }
    let mut cache = USERNAME.lock().unwrap();
    if cache.is_none() {
        let name = std::env::var("USER").or_else(|_| std::env::var("USERNAME"));
        let name = match name {
            Ok(n) if !n.is_empty() => n,
            _ => return 0, // FAIL
        };
        *cache = CString::new(name).ok();
    }
    let name_c = match cache.as_ref() {
        Some(n) => n,
        None => return 0,
    };
    let bytes = name_c.as_bytes_with_nul();
    let copy_len = ((len as usize).saturating_sub(1)).min(bytes.len() - 1);
    ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copy_len);
    *buf.add(copy_len) = 0;
    1 // OK
}

/// Free memory allocated by `get_user_name`.
#[no_mangle]
/// # Safety
/// May be called at any time to drop the cached user name.
pub unsafe extern "C" fn free_username() {
    let mut cache = USERNAME.lock().unwrap();
    *cache = None;
}

/// Representation of Vim's buffer structure with only fields used here.
#[repr(C)]
pub struct buf_T {
    pub b_p_ff: *const c_char,
    pub b_p_bin: c_int,
}

/// Representation of `exarg_T` with only fields used here.
#[repr(C)]
pub struct exarg_T {
    pub force_ff: c_int,
    pub force_bin: c_int,
}

/// Return the effective end-of-line type based on the buffer's 'fileformat'.
#[no_mangle]
/// # Safety
/// `buf` must be a valid pointer to a `buf_T` structure.
pub unsafe extern "C" fn get_fileformat(buf: *mut buf_T) -> c_int {
    if buf.is_null() {
        return EOL_UNIX;
    }
    let c = *(*buf).b_p_ff;
    if (*buf).b_p_bin != 0 || c == b'u' as c_char {
        EOL_UNIX
    } else if c == b'm' as c_char {
        EOL_MAC
    } else {
        EOL_DOS
    }
}

/// Like `get_fileformat` but overrides with command-line arguments when present.
#[no_mangle]
/// # Safety
/// `buf` and `eap` must be valid pointers when not null.
pub unsafe extern "C" fn get_fileformat_force(buf: *mut buf_T, eap: *mut exarg_T) -> c_int {
    let c: c_char = if !eap.is_null() && (*eap).force_ff != 0 {
        (*eap).force_ff as c_char
    } else {
        if (!eap.is_null() && (*eap).force_bin != 0 && (*eap).force_bin == FORCE_BIN)
            || (!buf.is_null() && (*buf).b_p_bin != 0)
        {
            return EOL_UNIX;
        }
        *(*buf).b_p_ff
    };
    if c == b'u' as c_char {
        EOL_UNIX
    } else if c == b'm' as c_char {
        EOL_MAC
    } else {
        EOL_DOS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_name_is_cached() {
        unsafe {
            std::env::set_var("USER", "vimtest");
            let mut buf = [0u8; 32];
            assert_eq!(get_user_name(buf.as_mut_ptr(), buf.len() as c_int), 1);
            assert_eq!(
                std::ffi::CStr::from_ptr(buf.as_ptr() as *const c_char)
                    .to_str()
                    .unwrap(),
                "vimtest"
            );

            std::env::set_var("USER", "changed");
            buf.fill(0);
            assert_eq!(get_user_name(buf.as_mut_ptr(), buf.len() as c_int), 1);
            // Name should still be the cached one
            assert_eq!(
                std::ffi::CStr::from_ptr(buf.as_ptr() as *const c_char)
                    .to_str()
                    .unwrap(),
                "vimtest"
            );

            free_username();
        }
    }

    #[test]
    fn get_fileformat_handles_options() {
        let ff_unix = CString::new("unix").unwrap();
        let ff_dos = CString::new("dos").unwrap();
        let mut buf = buf_T {
            b_p_ff: ff_unix.as_ptr(),
            b_p_bin: 0,
        };
        unsafe {
            assert_eq!(get_fileformat(&mut buf), EOL_UNIX);
            buf.b_p_ff = ff_dos.as_ptr();
            assert_eq!(get_fileformat(&mut buf), EOL_DOS);
            let mut ex = exarg_T {
                force_ff: 'm' as c_int,
                force_bin: 0,
            };
            assert_eq!(get_fileformat_force(&mut buf, &mut ex), EOL_MAC);
        }
    }
}
