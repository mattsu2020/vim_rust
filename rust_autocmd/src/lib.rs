use libc::{c_char, c_int};
use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::{Mutex, OnceLock};

#[repr(C)]
pub struct AutoCmd {
    pub cmd: *mut c_char,
    pub once: i8,
    pub nested: i8,
    pub last: i8,
    pub next: *mut AutoCmd,
}

#[repr(C)]
pub struct AutoPat {
    pub next: *mut AutoPat,
    pub pat: *mut c_char,
    pub cmds: *mut AutoCmd,
    pub group: c_int,
    pub patlen: c_int,
    pub buflocal_nr: c_int,
    pub allow_dirs: i8,
    pub last: i8,
}

static AUTOCMDS: OnceLock<Mutex<HashMap<c_int, Vec<String>>>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn rs_autocmd_register(
    event: c_int,
    _pat: *const c_char,
    cmd: *const c_char,
) -> c_int {
    if cmd.is_null() {
        return 0;
    }
    let cmd_str = unsafe { CStr::from_ptr(cmd) }.to_string_lossy().into_owned();
    AUTOCMDS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .entry(event)
        .or_default()
        .push(cmd_str);
    1
}

#[no_mangle]
pub extern "C" fn rs_autocmd_execute(event: c_int) -> c_int {
    if let Some(map) = AUTOCMDS.get() {
        let mut guard = map.lock().unwrap();
        if let Some(cmds) = guard.get_mut(&event) {
            let count = cmds.len() as c_int;
            cmds.clear();
            return count;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn register_and_execute() {
        let cmd = CString::new("echo 'hi'").unwrap();
        let pat = CString::new("*").unwrap();
        assert_eq!(rs_autocmd_register(1, pat.as_ptr(), cmd.as_ptr()), 1);
        assert_eq!(rs_autocmd_execute(1), 1);
        assert_eq!(rs_autocmd_execute(1), 0);
    }
}
