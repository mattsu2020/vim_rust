use std::ffi::{CStr, CString};
use std::os::raw::c_char;

mod linematch;
pub use linematch::{linematch_nbuffers, mmfile_t};

/// Compute a unified diff of two C strings.
///
/// # Safety
/// The input pointers must be valid C strings or null.
#[no_mangle]
pub extern "C" fn diff_unified(a: *const c_char, b: *const c_char) -> *mut c_char {
    unsafe {
        let a_str = if a.is_null() {
            ""
        } else {
            CStr::from_ptr(a).to_str().unwrap_or("")
        };
        let b_str = if b.is_null() {
            ""
        } else {
            CStr::from_ptr(b).to_str().unwrap_or("")
        };
        let diff = similar::TextDiff::from_lines(a_str, b_str)
            .unified_diff()
            .to_string();
        CString::new(diff).unwrap().into_raw()
    }
}

/// Free a string returned by [`diff_unified`].
#[no_mangle]
pub extern "C" fn diff_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int, c_long};

    #[test]
    fn diff_empty() {
        let a = CString::new("same\n").unwrap();
        let b = CString::new("same\n").unwrap();
        let res_ptr = diff_unified(a.as_ptr(), b.as_ptr());
        let res = unsafe { CStr::from_ptr(res_ptr).to_str().unwrap() };
        assert_eq!(res, "");
        diff_free(res_ptr);
    }

    #[test]
    fn diff_insert_delete() {
        let a = CString::new("a\nb\n").unwrap();
        let b = CString::new("a\nc\n").unwrap();
        let res_ptr = diff_unified(a.as_ptr(), b.as_ptr());
        let res = unsafe { CStr::from_ptr(res_ptr).to_str().unwrap().to_owned() };
        diff_free(res_ptr);
        assert!(res.contains("-b"));
        assert!(res.contains("+c"));
    }

    #[test]
    fn linematch_basic() {
        // Two simple buffers with one matching line and one differing line.
        let buf1 = b"a\nb\n";
        let buf2 = b"a\nc\n";
        let mf1 = mmfile_t {
            ptr: buf1.as_ptr() as *const c_char,
            size: buf1.len() as c_long,
        };
        let mf2 = mmfile_t {
            ptr: buf2.as_ptr() as *const c_char,
            size: buf2.len() as c_long,
        };
        let blocks = vec![&mf1 as *const mmfile_t, &mf2 as *const mmfile_t];
        let lens = vec![2 as c_int, 2 as c_int];
        let mut decisions_ptr: *mut c_int = std::ptr::null_mut();
        let n = linematch_nbuffers(blocks.as_ptr(), lens.as_ptr(), 2, &mut decisions_ptr, 0);
        assert_eq!(n, 3);
        let decisions = unsafe { std::slice::from_raw_parts(decisions_ptr, n) };
        assert_eq!(decisions[0], 3); // match first line
        assert!(decisions[1] == 1 || decisions[1] == 2); // skip one of the differing lines
        assert!(decisions[2] == 1 || decisions[2] == 2); // skip the other line
        unsafe {
            libc::free(decisions_ptr as *mut libc::c_void);
        }
    }
}
