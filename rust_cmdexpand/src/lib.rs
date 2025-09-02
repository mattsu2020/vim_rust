use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::OnceLock;

// A minimal list of Ex commands used for expansion.
static COMMANDS: OnceLock<Vec<&'static str>> = OnceLock::new();

/// Initialize the command table.  This would normally mirror Vim's built-in
/// commands but is kept very small here for demonstration purposes.
#[no_mangle]
pub extern "C" fn cmdexpand_init() {
    COMMANDS.get_or_init(|| vec!["quit", "write", "wq", "help"]);
}

/// Return a space separated list of commands that start with `prefix`.
/// The returned string is allocated with `CString::into_raw` and must be
/// freed by calling [`cmdexpand_free`].
#[no_mangle]
pub extern "C" fn cmdexpand(prefix: *const c_char) -> *mut c_char {
    if prefix.is_null() {
        return std::ptr::null_mut();
    }
    let p = unsafe { CStr::from_ptr(prefix) }.to_str().unwrap_or("");
    let cmds = COMMANDS.get_or_init(|| vec![]);
    let matches: Vec<&str> = cmds.iter().copied().filter(|c| c.starts_with(p)).collect();
    let joined = matches.join(" ");
    CString::new(joined).unwrap().into_raw()
}

/// Free a string previously returned by [`cmdexpand`].
#[no_mangle]
pub extern "C" fn cmdexpand_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn expand_commands() {
        cmdexpand_init();
        let prefix = CString::new("w").unwrap();
        let res = cmdexpand(prefix.as_ptr());
        let s = unsafe { CStr::from_ptr(res) }.to_str().unwrap().to_string();
        cmdexpand_free(res);
        assert_eq!(s, "write wq");
    }
}

