use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use rust_excmds::{do_cmdline, CharU};

/// Errors that can occur while executing Ex commands.
#[derive(Debug, PartialEq)]
pub enum ExEvalError {
    /// The provided command line was empty.
    EmptyCommand,
    /// The command could not be executed.
    ExecutionFailed,
    /// Invalid UTF-8 in the command line.
    InvalidEncoding,
}

/// Execute a single Ex command line.
///
/// This is a small Rust wrapper around `rust_excmds::do_cmdline` that uses
/// `Result` for error reporting.
pub fn execute_command(line: &str) -> Result<(), ExEvalError> {
    if line.trim().is_empty() {
        return Err(ExEvalError::EmptyCommand);
    }
    let cstring = CString::new(line).map_err(|_| ExEvalError::InvalidEncoding)?;
    let mut bytes = cstring.into_bytes_with_nul();
    let ptr = bytes.as_mut_ptr() as *mut CharU;
    let ret = unsafe { do_cmdline(ptr, None, std::ptr::null_mut(), 0) };
    if ret == 0 {
        Ok(())
    } else {
        Err(ExEvalError::ExecutionFailed)
    }
}

/// Read command lines from the supplied iterator and execute them.
///
/// Each item from the iterator represents one command line.  If an empty
/// command is encountered, an `ExEvalError::EmptyCommand` is returned.
pub fn eval_from_iter<I>(iter: I) -> Result<(), ExEvalError>
where
    I: IntoIterator<Item = String>,
{
    for line in iter {
        execute_command(&line)?;
    }
    Ok(())
}

/// FFI wrapper for executing a single command line.
///
/// Returns 0 on success and non-zero on failure.
#[no_mangle]
pub extern "C" fn rs_exeval_execute(line: *const c_char) -> c_int {
    if line.is_null() {
        return 1;
    }
    let c_str = unsafe { CStr::from_ptr(line) };
    match c_str.to_str() {
        Ok(s) => match execute_command(s) {
            Ok(()) => 0,
            Err(_) => 1,
        },
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_non_empty() {
        assert!(execute_command("cmd1|cmd2").is_ok());
    }

    #[test]
    fn reject_empty() {
        assert_eq!(execute_command("   "), Err(ExEvalError::EmptyCommand));
    }

    #[test]
    fn eval_multiple_lines() {
        let cmds = vec!["cmd1".to_string(), "cmd2".to_string()];
        assert!(eval_from_iter(cmds).is_ok());
    }
}
