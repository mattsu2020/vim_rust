use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::process::{Child, Command};
use std::ptr;

/// Minimal job representation wrapping a spawned child process.
pub struct Job {
    child: Child,
}

/// Start a job using the command provided as a C string.  The command is
/// executed using the system shell when available.  On success an opaque pointer
/// to the Job is returned, otherwise `NULL` is returned.
#[no_mangle]
pub extern "C" fn job_start(cmd: *const c_char) -> *mut Job {
    if cmd.is_null() {
        return ptr::null_mut();
    }
    let cstr = unsafe { CStr::from_ptr(cmd) };
    let cmd_str = match cstr.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    // Spawn using the platform shell for parity with Vim's job_start
    #[cfg(unix)]
    let child = Command::new("sh").arg("-c").arg(cmd_str).spawn();
    #[cfg(windows)]
    let child = Command::new("cmd").arg("/C").arg(cmd_str).spawn();

    match child {
        Ok(child) => Box::into_raw(Box::new(Job { child })),
        Err(_) => ptr::null_mut(),
    }
}

/// Attempt to terminate the job.  Returns 0 on success and -1 on failure.
#[no_mangle]
pub extern "C" fn job_stop(job: *mut Job) -> c_int {
    if job.is_null() {
        return -1;
    }
    let job = unsafe { &mut *job };
    match job.child.kill() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Check the status of the job.  Returns 0 if running, the exit code if the
/// process has finished, and -1 on error.
#[no_mangle]
pub extern "C" fn job_status(job: *mut Job) -> c_int {
    if job.is_null() {
        return -1;
    }
    let job = unsafe { &mut *job };
    match job.child.try_wait() {
        Ok(Some(status)) => status.code().unwrap_or(-1),
        Ok(None) => 0,
        Err(_) => -1,
    }
}
