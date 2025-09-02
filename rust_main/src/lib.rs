use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::sync::{Mutex, OnceLock};

// Stores commands processed through the FFI interface.
static COMMAND_LOG: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
// Counts how many iterations the event loop has executed.
static EVENT_COUNTER: OnceLock<Mutex<c_int>> = OnceLock::new();

/// Initialize the minimal state used by the Rust main module.
#[no_mangle]
pub extern "C" fn vim_init() {
    COMMAND_LOG.get_or_init(|| Mutex::new(Vec::new()));
    EVENT_COUNTER.get_or_init(|| Mutex::new(0));
}

/// Record a command string.  Returns the number of stored commands or -1 on error.
#[no_mangle]
pub extern "C" fn vim_process_command(cmd: *const c_char) -> c_int {
    if cmd.is_null() {
        return -1;
    }
    let s = unsafe { CStr::from_ptr(cmd) };
    let cmd = s.to_string_lossy().into_owned();
    let mut log = COMMAND_LOG.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap();
    log.push(cmd);
    log.len() as c_int
}

/// Simulate the Vim event loop for `iterations` steps.
/// Returns the total number of iterations executed so far.
#[no_mangle]
pub extern "C" fn vim_event_loop(iterations: c_int) -> c_int {
    let mut count = EVENT_COUNTER
        .get_or_init(|| Mutex::new(0))
        .lock()
        .unwrap();
    for _ in 0..iterations {
        *count += 1;
    }
    *count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn init_and_process() {
        vim_init();
        let cmd = CString::new("echo test").unwrap();
        let n = vim_process_command(cmd.as_ptr());
        assert_eq!(n, 1);
    }

    #[test]
    fn run_event_loop() {
        vim_init();
        let total = vim_event_loop(3);
        assert_eq!(total, 3);
        let total = vim_event_loop(2);
        assert_eq!(total, 5);
    }
}

