use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::process::Command;

fn run_cscope(query: *const c_char) -> Result<(), ()> {
    if query.is_null() {
        return Err(());
    }
    let q = unsafe { CStr::from_ptr(query) }.to_str().map_err(|_| ())?;
    let status = Command::new("cscope").arg("-L").arg(q).status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(()),
    }
}

/// Query cscope using the provided query string.
/// Returns 1 on success, 0 on failure.
#[no_mangle]
pub extern "C" fn vim_cscope_query(query: *const c_char) -> c_int {
    run_cscope(query).map_or(0, |_| 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn null_query_fails() {
        assert_eq!(vim_cscope_query(std::ptr::null()), 0);
    }

    #[test]
    fn non_null_query_does_not_crash() {
        let q = CString::new("hello").unwrap();
        let _ = vim_cscope_query(q.as_ptr());
    }
}
