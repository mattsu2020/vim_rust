use once_cell::sync::OnceCell;
use pyo3::prelude::*;
use std::os::raw::{c_char, c_int};

use rust_ffi::{cstr_to_str, result_to_int, FFIError, FFIResult};

static PY_INITIALIZED: OnceCell<()> = OnceCell::new();

fn init() -> FFIResult<()> {
    PY_INITIALIZED
        .get_or_init(|| {
            pyo3::prepare_freethreaded_python();
        });
    Ok(())
}

fn end() -> FFIResult<()> {
    // Finalization is a no-op to avoid interpreter shutdown issues in tests.
    Ok(())
}

fn run_code(code: *const c_char) -> FFIResult<()> {
    init()?;
    let source = cstr_to_str(code)?;
    Python::with_gil(|py| py.run(source, None, None)).map_err(|_| FFIError::Exec)
}

/// Initialize the embedded Python interpreter for use within Vim.
/// Returns 1 on success to mimic Vim's boolean conventions.
#[no_mangle]
pub extern "C" fn vim_python_init() -> c_int {
    result_to_int(init())
}

/// Execute a snippet of Python code provided as a C string.
/// Returns 1 when the code runs successfully, 0 otherwise.
#[no_mangle]
pub extern "C" fn vim_python_exec(code: *const c_char) -> c_int {
    result_to_int(run_code(code))
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

/// Finalize the embedded Python interpreter.
#[no_mangle]
pub extern "C" fn vim_python_end() -> c_int {
    result_to_int(end())
}

/// Alias to finalize Python via the traditional name.
#[no_mangle]
pub extern "C" fn python_end() -> c_int {
    vim_python_end()
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
        assert_eq!(vim_python_end(), 1);
    }

    #[test]
    fn null_exec_returns_error() {
        assert_eq!(vim_python_exec(std::ptr::null()), 0);
    }
}
