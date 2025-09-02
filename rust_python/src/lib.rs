use pyo3::prelude::*;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// Initialize the embedded Python interpreter for use within Vim.
/// Returns 1 on success to mimic Vim's boolean conventions.
#[no_mangle]
pub extern "C" fn vim_python_init() -> c_int {
    pyo3::prepare_freethreaded_python();
    1
}

/// Execute a snippet of Python code provided as a C string.
/// Returns 1 when the code runs successfully, 0 otherwise.
#[no_mangle]
pub extern "C" fn vim_python_exec(code: *const c_char) -> c_int {
    if code.is_null() {
        return 0;
    }
    let c_str = unsafe { CStr::from_ptr(code) };
    let source = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    Python::with_gil(|py| match py.run(source, None, None) {
        Ok(_) => 1,
        Err(_) => 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn init_and_exec() {
        assert_eq!(vim_python_init(), 1);
        let code = CString::new("x = 1 + 1").unwrap();
        assert_eq!(vim_python_exec(code.as_ptr()), 1);
    }
}
