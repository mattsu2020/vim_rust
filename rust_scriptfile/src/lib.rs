use std::fs;
use std::ffi::CStr;
use std::os::raw::c_char;

use rust_ex_eval::ex_eval;
use rust_ex_getln::get_line;

#[derive(Debug)]
pub enum ScriptError {
    Io,
    Eval,
    InvalidEncoding,
}

pub fn exec_script(path: &str) -> Result<(), ScriptError> {
    let content = fs::read_to_string(path).map_err(|_| ScriptError::Io)?;
    let mut idx = 0;
    while let Some(line) = get_line(&content, idx) {
        ex_eval(&line).map_err(|_| ScriptError::Eval)?;
        idx += 1;
    }
    Ok(())
}

#[no_mangle]
pub extern "C" fn rs_exec_scriptfile(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let c_path = unsafe { CStr::from_ptr(path) };
    match c_path.to_str() {
        Ok(p) => exec_script(p).is_ok(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn execute_scriptfile() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "cmd1").unwrap();
        writeln!(file, "cmd2").unwrap();
        assert!(exec_script(file.path().to_str().unwrap()).is_ok());
    }
}

