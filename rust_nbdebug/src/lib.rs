use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::{env, thread, time::Duration};

static NB_DEBUG_FILE: Lazy<Mutex<Option<std::fs::File>>> = Lazy::new(|| Mutex::new(None));
static NB_DLEVEL: AtomicU32 = AtomicU32::new(0);

const NB_TRACE: u32 = 0x00000001;
const WT_ENV: u32 = 1;
const WT_WAIT: u32 = 2;
const WT_STOP: u32 = 3;

/// Internal helper to log a message if debugging is enabled.
pub fn nbdbg(msg: &str) {
    if NB_DLEVEL.load(Ordering::Relaxed) & NB_TRACE != 0 {
        if let Ok(mut guard) = NB_DEBUG_FILE.lock() {
            if let Some(f) = guard.as_mut() {
                let _ = writeln!(f, "{}", msg);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rs_nbdbg(msg: *const c_char) {
    if msg.is_null() {
        return;
    }
    if let Ok(s) = CStr::from_ptr(msg).to_str() {
        nbdbg(s);
    }
}

#[no_mangle]
pub unsafe extern "C" fn rs_nbdebug_log_init(log_var: *const c_char, level_var: *const c_char) {
    let log_path = if !log_var.is_null() {
        let var = CStr::from_ptr(log_var).to_string_lossy().to_string();
        env::var(var).ok()
    } else {
        None
    };

    if let Some(path) = log_path {
        if let Ok(file) = OpenOptions::new().create(true).append(true).open(&path) {
            if let Ok(mut guard) = NB_DEBUG_FILE.lock() {
                *guard = Some(file);
            }
        }
        let level = if !level_var.is_null() {
            let var = CStr::from_ptr(level_var).to_string_lossy().to_string();
            env::var(var).ok()
        } else {
            None
        };
        let lvl = level
            .and_then(|s| u32::from_str_radix(&s, 0).ok())
            .unwrap_or(NB_TRACE);
        NB_DLEVEL.store(lvl, Ordering::Relaxed);
    }
}

#[no_mangle]
pub unsafe extern "C" fn rs_nbdebug_wait(wait_flags: u32, wait_var: *const c_char, wait_secs: u32) {
    // Determine the number of seconds to wait based on flags and environment.
    if (wait_flags & WT_ENV) != 0 && !wait_var.is_null() {
        if let Ok(var) = CStr::from_ptr(wait_var).to_str() {
            if let Ok(val) = env::var(var) {
                if let Ok(secs) = val.parse::<u64>() {
                    thread::sleep(Duration::from_secs(secs));
                    return;
                }
            }
        }
    }
    if (wait_flags & WT_WAIT) != 0 {
        if let Some(home) = env::var_os("HOME") {
            let mut path = PathBuf::from(home);
            path.push(".gvimwait");
            if path.exists() {
                let secs = wait_secs.min(120).max(1) as u64;
                thread::sleep(Duration::from_secs(secs));
                return;
            }
        }
    }
    if (wait_flags & WT_STOP) != 0 {
        if let Some(home) = env::var_os("HOME") {
            let mut path = PathBuf::from(home);
            path.push(".gvimstop");
            if path.exists() {
                loop {
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn log_messages() {
        let dir = env::temp_dir();
        let file_path = dir.join("nbdebug_test.log");
        if let Ok(file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
        {
            if let Ok(mut guard) = NB_DEBUG_FILE.lock() {
                *guard = Some(file);
            }
        }
        NB_DLEVEL.store(1, Ordering::Relaxed);
        nbdbg("hello");
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("hello"));
        let _ = fs::remove_file(file_path);
    }
}
