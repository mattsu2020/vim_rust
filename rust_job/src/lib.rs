use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::process::Command;

/// Run `command` using the system shell.
/// Returns the exit code on success, or -1 on error.
#[no_mangle]
pub extern "C" fn rs_run_job(command: *const c_char) -> c_int {
    if command.is_null() {
        return -1;
    }
    let c_cmd = unsafe { CStr::from_ptr(command) };
    let cmd_str = match c_cmd.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd_str]).status()
    } else {
        Command::new("sh").arg("-c").arg(cmd_str).status()
    };
    match status {
        Ok(s) => s.code().unwrap_or(-1),
        Err(_) => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn run_echo_command() {
        let cmd = CString::new("echo rust_job").unwrap();
        let code = rs_run_job(cmd.as_ptr());
        assert_eq!(code, 0);
    }
}
