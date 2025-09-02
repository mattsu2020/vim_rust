use libc::{c_char, c_int};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::Mutex;

struct AutoCmd {
    cmd: CString,
    once: bool,
    nested: bool,
}

struct AutoPat {
    pattern: String,
    regex: Regex,
    cmds: Vec<AutoCmd>,
}

type Event = i32;

static AUTOCMDS: Lazy<Mutex<HashMap<Event, Vec<AutoPat>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[no_mangle]
pub extern "C" fn rust_autocmd_clear() {
    AUTOCMDS.lock().unwrap().clear();
}

unsafe fn to_str(ptr: *const c_char) -> Option<&'static str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}

#[no_mangle]
pub extern "C" fn rust_autocmd_add(
    event: c_int,
    pat: *const c_char,
    cmd: *const c_char,
    once: c_int,
    nested: c_int,
) -> c_int {
    let pat_str = match unsafe { to_str(pat) } {
        Some(s) => s.to_owned(),
        None => return 0,
    };
    let cmd_str = match unsafe { to_str(cmd) } {
        Some(s) => s,
        None => return 0,
    };
    let re = match Regex::new(&pat_str) {
        Ok(r) => r,
        Err(_) => return 0,
    };

    let acmd = AutoCmd {
        cmd: CString::new(cmd_str).unwrap(),
        once: once != 0,
        nested: nested != 0,
    };
    let mut map = AUTOCMDS.lock().unwrap();
    let list = map.entry(event).or_insert_with(Vec::new);

    if let Some(ap) = list.iter_mut().find(|ap| ap.pattern == pat_str) {
        ap.cmds.push(acmd);
    } else {
        list.push(AutoPat {
            pattern: pat_str,
            regex: re,
            cmds: vec![acmd],
        });
    }
    1
}

#[no_mangle]
pub extern "C" fn rust_autocmd_do(event: c_int, name: *const c_char) -> c_int {
    let name = match unsafe { to_str(name) } {
        Some(s) => s,
        None => return 0,
    };
    let mut map = AUTOCMDS.lock().unwrap();
    let mut done = 0;
    if let Some(list) = map.get_mut(&event) {
        for ap in list.iter_mut() {
            if ap.regex.is_match(name) {
                done += ap.cmds.len() as c_int;
                if ap.cmds.iter().any(|c| c.once) {
                    ap.cmds.retain(|c| !c.once);
                }
            }
        }
        list.retain(|ap| !ap.cmds.is_empty());
    }
    done
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn basic_match() {
        rust_autocmd_clear();
        let pat = CString::new("^foo").unwrap();
        let cmd = CString::new("echo foo").unwrap();
        assert_eq!(rust_autocmd_add(1, pat.as_ptr(), cmd.as_ptr(), 0, 0), 1);
        let name = CString::new("foobar").unwrap();
        assert_eq!(rust_autocmd_do(1, name.as_ptr()), 1);
        // second trigger should still find command since once=0
        assert_eq!(rust_autocmd_do(1, name.as_ptr()), 1);
    }

    #[test]
    fn once_clears_command() {
        rust_autocmd_clear();
        let pat = CString::new("bar$").unwrap();
        let cmd = CString::new("echo bar").unwrap();
        assert_eq!(rust_autocmd_add(2, pat.as_ptr(), cmd.as_ptr(), 1, 0), 1);
        let name = CString::new("mybar").unwrap();
        assert_eq!(rust_autocmd_do(2, name.as_ptr()), 1);
        // command removed after once
        assert_eq!(rust_autocmd_do(2, name.as_ptr()), 0);
    }
}
