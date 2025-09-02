use std::ffi::{CString, CStr};
use std::thread::sleep;
use std::time::Duration;

use vim_job::{job_start, job_status, job_stop, job_stdout, job_free_string};

#[test]
fn job_output_and_status() {
    let cmd = if cfg!(unix) { "echo hello" } else { "cmd /C echo hello" };
    let ccmd = CString::new(cmd).unwrap();
    let job = job_start(ccmd.as_ptr());
    assert!(!job.is_null());

    loop {
        let st = job_status(job);
        if st != -1 {
            assert_eq!(st, 0);
            break;
        }
        sleep(Duration::from_millis(10));
    }

    let out_ptr = job_stdout(job);
    assert!(!out_ptr.is_null());
    let out = unsafe { CStr::from_ptr(out_ptr) }.to_str().unwrap().trim().to_string();
    job_free_string(out_ptr);
    assert_eq!(out, "hello");
}

#[test]
fn job_can_be_stopped() {
    let cmd = if cfg!(unix) { "sleep 30" } else { "ping -n 30 127.0.0.1 >NUL" };
    let ccmd = CString::new(cmd).unwrap();
    let job = job_start(ccmd.as_ptr());
    assert!(!job.is_null());
    assert_eq!(job_status(job), -1);
    assert_eq!(job_stop(job), 0);
}
