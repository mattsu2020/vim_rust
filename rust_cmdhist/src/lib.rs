use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;

pub use rust_usercmd::{rs_user_command_delete, rs_user_command_register};

const HIST_CMD: usize = 0;
const HIST_SEARCH: usize = 1;
const HIST_EXPR: usize = 2;
const HIST_INPUT: usize = 3;
const HIST_DEBUG: usize = 4;
const HIST_COUNT: usize = 5;
const DEFAULT_HISTORY_LEN: usize = 50;

#[derive(Clone, Default)]
struct HistEntry {
    hisnum: i32,
    viminfo: bool,
    hisstr: String,
    hisstrlen: usize,
    time_set: i64,
}

struct HistoryState {
    history: [Vec<HistEntry>; HIST_COUNT],
    hisidx: [i32; HIST_COUNT],
    hisnum: [i32; HIST_COUNT],
    hislen: usize,
}

impl HistoryState {
    fn new() -> Self {
        Self {
            history: std::array::from_fn(|_| Vec::new()),
            hisidx: [-1; HIST_COUNT],
            hisnum: [0; HIST_COUNT],
            hislen: DEFAULT_HISTORY_LEN,
        }
    }

    fn init(&mut self, len: usize) {
        self.hislen = len;
        for vec in &mut self.history {
            vec.clear();
        }
        self.hisidx = [-1; HIST_COUNT];
        self.hisnum = [0; HIST_COUNT];
    }

    fn add_to_history(&mut self, histype: usize, entry: &str) {
        if self.hislen == 0 {
            return;
        }
        let hist = &mut self.history[histype];
        if let Some(pos) = hist.iter().position(|h| h.hisstr == entry) {
            let item = hist.remove(pos);
            hist.push(item);
        } else {
            if hist.len() == self.hislen {
                hist.remove(0);
            }
            self.hisnum[histype] += 1;
            hist.push(HistEntry {
                hisnum: self.hisnum[histype],
                viminfo: false,
                hisstrlen: entry.len(),
                hisstr: entry.to_string(),
                time_set: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            });
        }
        self.hisidx[histype] = (hist.len() - 1) as i32;
    }
}

static STATE: Lazy<Mutex<HistoryState>> = Lazy::new(|| Mutex::new(HistoryState::new()));

pub fn cmd_history_init(len: c_int) {
    let mut state = STATE.lock().unwrap();
    state.init(len as usize);
}

pub fn cmd_history_add(cmd: *const c_char) {
    if cmd.is_null() {
        return;
    }
    let cmd = unsafe { CStr::from_ptr(cmd) }
        .to_string_lossy()
        .into_owned();
    let mut state = STATE.lock().unwrap();
    state.add_to_history(HIST_CMD, &cmd);
}

pub fn cmd_history_get(idx: c_int) -> *const c_char {
    let state = STATE.lock().unwrap();
    if let Some(entry) = state.history[HIST_CMD].get(idx as usize) {
        CString::new(entry.hisstr.clone()).unwrap().into_raw()
    } else {
        std::ptr::null()
    }
}

pub fn cmd_history_len() -> c_int {
    let state = STATE.lock().unwrap();
    state.history[HIST_CMD].len() as c_int
}

pub fn cmd_history_clear() {
    let mut state = STATE.lock().unwrap();
    let len = state.hislen;
    state.init(len);
}

#[cfg(feature = "ffi")]
pub mod ffi {
    use super::*;

    #[no_mangle]
    pub extern "C" fn rs_cmd_history_init(len: c_int) {
        cmd_history_init(len);
    }

    #[no_mangle]
    pub extern "C" fn rs_cmd_history_add(cmd: *const c_char) {
        cmd_history_add(cmd);
    }

    #[no_mangle]
    pub extern "C" fn rs_cmd_history_get(idx: c_int) -> *const c_char {
        cmd_history_get(idx)
    }

    #[no_mangle]
    pub extern "C" fn rs_cmd_history_len() -> c_int {
        cmd_history_len()
    }

    #[no_mangle]
    pub extern "C" fn rs_cmd_history_clear() {
        cmd_history_clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn add_and_get() {
        cmd_history_clear();
        cmd_history_add(b"cmd1\0".as_ptr() as *const c_char);
        let ptr = cmd_history_get(0);
        let cstr = unsafe { CStr::from_ptr(ptr) };
        assert_eq!(cstr.to_str().unwrap(), "cmd1");
    }

    #[test]
    fn limit_and_duplicates() {
        cmd_history_init(2);
        cmd_history_add(b"cmd1\0".as_ptr() as *const c_char);
        cmd_history_add(b"cmd2\0".as_ptr() as *const c_char);
        cmd_history_add(b"cmd3\0".as_ptr() as *const c_char); // cmd1 dropped
        let first = unsafe { CStr::from_ptr(cmd_history_get(0)) };
        assert_eq!(first.to_str().unwrap(), "cmd2");
        cmd_history_add(b"cmd2\0".as_ptr() as *const c_char); // move cmd2 to end
        let last = unsafe { CStr::from_ptr(cmd_history_get(1)) };
        assert_eq!(last.to_str().unwrap(), "cmd2");
    }

    #[test]
    fn register_and_delete_command() {
        let name = CString::new("MyCmd").unwrap();
        let rep = CString::new("echo").unwrap();
        let res = rs_user_command_register(
            name.as_ptr(),
            rep.as_ptr(),
            0,
            0,
            0,
            0,
            std::ptr::null(),
            0,
            0,
        );
        assert_eq!(res, 0);
        assert_eq!(rs_user_command_delete(name.as_ptr()), 0);
    }
}
