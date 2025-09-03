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

fn ptr_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr).to_str().ok() }
}

#[no_mangle]
pub extern "C" fn rust_autocmd_do(event: c_int, name: *const c_char) -> c_int {
    let name = match ptr_to_str(name) {
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
