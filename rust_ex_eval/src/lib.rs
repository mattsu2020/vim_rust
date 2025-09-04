use rust_exeval::{execute_command, ExEvalError};
use rust_usercmd::expand_user_command;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

pub fn ex_eval(line: &str) -> Result<(), ExEvalError> {
    let expanded = expand_user_command(line).unwrap_or_else(|| line.to_string());
    execute_command(&expanded)
}

#[no_mangle]
pub extern "C" fn rs_ex_eval(line: *const c_char) -> c_int {
    if line.is_null() {
        return 1;
    }
    let c_str = unsafe { CStr::from_ptr(line) };
    match c_str.to_str() {
        Ok(s) => match ex_eval(s) {
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
    fn eval_simple_command() {
        assert!(ex_eval("cmd1").is_ok());
    }

    #[test]
    fn eval_user_command() {
        rust_usercmd::define_user_command("Foo", "cmd1");
        assert!(ex_eval("Foo").is_ok());
    }
}

