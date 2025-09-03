use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long, c_void};
use similar::{Algorithm, TextDiff};
use diff::{mmfile_t, mmbuffer_t, xpparam_t, xdemitcb_t, xdemitconf_t,
    xdl_diff as rs_xdl_diff, xdiff_out_unified as rs_xdiff_out_unified,
    xdiff_out_indices as rs_xdiff_out_indices};

fn diff_internal(a: &str, b: &str) -> String {
    TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .diff_lines(a, b)
        .unified_diff()
        .to_string()
}

#[no_mangle]
pub extern "C" fn vim_xdiff_diff(a: *const c_char, b: *const c_char) -> *mut c_char {
    if a.is_null() || b.is_null() {
        return std::ptr::null_mut();
    }
    let a_str = unsafe { CStr::from_ptr(a) }.to_string_lossy().into_owned();
    let b_str = unsafe { CStr::from_ptr(b) }.to_string_lossy().into_owned();
    let result = diff_internal(&a_str, &b_str);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn vim_xdiff_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr)); }
    }
}

// Re-export the xdiff API used by src/diff.c, forwarding to the Rust implementation
// provided by the `diff` crate.  This allows the C code to link against the Rust
// diff algorithm instead of the former C implementation.

#[no_mangle]
pub unsafe extern "C" fn xdl_diff(
    mf1: *const mmfile_t,
    mf2: *const mmfile_t,
    xpp: *const xpparam_t,
    xecfg: *const xdemitconf_t,
    ecb: *mut xdemitcb_t,
) -> c_int {
    rs_xdl_diff(mf1, mf2, xpp, xecfg, ecb)
}

#[no_mangle]
pub unsafe extern "C" fn xdiff_out_unified(
    priv_: *mut c_void,
    mb: *mut mmbuffer_t,
    nbuf: c_int,
) -> c_int {
    rs_xdiff_out_unified(priv_, mb, nbuf)
}

#[no_mangle]
pub unsafe extern "C" fn xdiff_out_indices(
    start_a: c_long,
    count_a: c_long,
    start_b: c_long,
    count_b: c_long,
    priv_: *mut c_void,
) -> c_int {
    rs_xdiff_out_indices(start_a, count_a, start_b, count_b, priv_)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ffi::CStr;
    use std::time::Instant;

    #[test]
    fn diff_basic() {
        let a = "a\nb\n";
        let b = "a\nc\n";
        let diff = diff_internal(a, b);
        assert!(diff.contains("-b"));
        assert!(diff.contains("+c"));
    }

    #[test]
    fn ffi_roundtrip() {
        let a = CString::new("one\ntwo\n").unwrap();
        let b = CString::new("one\nthree\n").unwrap();
        let ptr = vim_xdiff_diff(a.as_ptr(), b.as_ptr());
        let diff = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        vim_xdiff_free(ptr);
        assert!(diff.contains("-two"));
        assert!(diff.contains("+three"));
    }

    #[test]
    fn performance_small() {
        let a = "a\n".repeat(1000);
        let mut b = "a\n".repeat(999);
        b.push_str("b\n");
        let start = Instant::now();
        let _ = diff_internal(&a, &b);
        // ensure the diff runs quickly for small inputs
        assert!(start.elapsed().as_millis() < 200);
    }

    #[test]
    fn performance_large() {
        let a = "a\n".repeat(10_000);
        let mut b = "a\n".repeat(9_999);
        b.push_str("b\n");
        let start = Instant::now();
        let _ = diff_internal(&a, &b);
        // basic performance check for larger inputs
        assert!(start.elapsed().as_millis() < 2000);
    }
}
