use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::{Arc, Mutex};

use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::runtime::Runtime;
use std::process::Stdio;
use tokio::task::JoinHandle;

/// Minimal job representation wrapping a spawned child process.
pub struct Job {
    child: Child,
    stdout: Arc<Mutex<Vec<u8>>>,
    stderr: Arc<Mutex<Vec<u8>>>,
    rt: Runtime,
    stdout_task: Option<JoinHandle<()>>,
    stderr_task: Option<JoinHandle<()>>,
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
    let mut command = Command::new("sh");
    #[cfg(windows)]
    let mut command = Command::new("cmd");

    #[cfg(unix)]
    let command = command.arg("-c").arg(cmd_str);
    #[cfg(windows)]
    let command = command.arg("/C").arg(cmd_str);

    let command = command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    let child_res = {
        let _g = rt.enter();
        command.spawn()
    };

    match child_res {
        Ok(mut child) => {
            let stdout_buf = Arc::new(Mutex::new(Vec::new()));
            let stderr_buf = Arc::new(Mutex::new(Vec::new()));

            let stdout_task = if let Some(mut stdout) = child.stdout.take() {
                let buf = Arc::clone(&stdout_buf);
                Some(rt.spawn(async move {
                    let mut data = Vec::new();
                    let _ = stdout.read_to_end(&mut data).await;
                    if let Ok(mut locked) = buf.lock() {
                        *locked = data;
                    }
                }))
            } else {
                None
            };

            let stderr_task = if let Some(mut stderr) = child.stderr.take() {
                let buf = Arc::clone(&stderr_buf);
                Some(rt.spawn(async move {
                    let mut data = Vec::new();
                    let _ = stderr.read_to_end(&mut data).await;
                    if let Ok(mut locked) = buf.lock() {
                        *locked = data;
                    }
                }))
            } else {
                None
            };

            Box::into_raw(Box::new(Job { child, stdout: stdout_buf, stderr: stderr_buf, rt, stdout_task, stderr_task }))
        }
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
    match job.rt.block_on(job.child.kill()) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Check the status of the job.  Returns -1 if still running, the exit code if
/// the process has finished, and -2 on error.
#[no_mangle]
pub extern "C" fn job_status(job: *mut Job) -> c_int {
    if job.is_null() {
        return -2;
    }
    let job = unsafe { &mut *job };
    match job.child.try_wait() {
        Ok(Some(status)) => status.code().unwrap_or(-1),
        Ok(None) => -1,
        Err(_) => -2,
    }
}

/// Retrieve the captured standard output of the job as a newly allocated C
/// string.  The caller must free the returned pointer using `job_free_string`.
#[no_mangle]
pub extern "C" fn job_stdout(job: *mut Job) -> *mut c_char {
    if job.is_null() {
        return ptr::null_mut();
    }
    let job = unsafe { &mut *job };
    if let Some(handle) = job.stdout_task.take() {
        let _ = job.rt.block_on(handle);
    }
    let data = job.stdout.lock().unwrap();
    match CString::new(data.clone()) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Retrieve the captured standard error of the job as a newly allocated C
/// string.  The caller must free the returned pointer using `job_free_string`.
#[no_mangle]
pub extern "C" fn job_stderr(job: *mut Job) -> *mut c_char {
    if job.is_null() {
        return ptr::null_mut();
    }
    let job = unsafe { &mut *job };
    if let Some(handle) = job.stderr_task.take() {
        let _ = job.rt.block_on(handle);
    }
    let data = job.stderr.lock().unwrap();
    match CString::new(data.clone()) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Free a string previously returned by `job_stdout` or `job_stderr`.
#[no_mangle]
pub extern "C" fn job_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe { let _ = CString::from_raw(s); }
}
