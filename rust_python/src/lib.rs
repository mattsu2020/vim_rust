use pyo3::prelude::*;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

fn init() -> Result<(), ()> {
    pyo3::prepare_freethreaded_python();
    Ok(())
}

fn run_code(code: *const c_char) -> Result<(), ()> {
    if code.is_null() {
        return Err(());
    }
    let source = unsafe { CStr::from_ptr(code) }.to_str().map_err(|_| ())?;
    Python::with_gil(|py| py.run(source, None, None)).map_err(|_| ())
}

/// Initialize the embedded Python interpreter for use within Vim.
/// Returns 1 on success to mimic Vim's boolean conventions.
#[no_mangle]
pub extern "C" fn vim_python_init() -> c_int {
    init().map_or(0, |_| 1)
}

/// Execute a snippet of Python code provided as a C string.
/// Returns 1 when the code runs successfully, 0 otherwise.
#[no_mangle]
pub extern "C" fn vim_python_exec(code: *const c_char) -> c_int {
    run_code(code).map_or(0, |_| 1)
}

/// Alias exposed for compatibility with existing Vim commands.
#[no_mangle]
pub extern "C" fn python_execute(code: *const c_char) -> c_int {
    vim_python_exec(code)
}

/// Alias to initialize Python via a traditional name.
#[no_mangle]
pub extern "C" fn python_init() -> c_int {
    vim_python_init()
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
