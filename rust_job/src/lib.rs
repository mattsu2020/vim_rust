use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use tokio::process::Command;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct JobConfig {
    pub cmd: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum JobError {
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn run_job(config: JobConfig) -> Result<i32, JobError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        eprintln!("failed to create runtime: {}", e);
        e
    })?;
    let status = rt
        .block_on(async { Command::new(&config.cmd).args(&config.args).status().await })
        .map_err(|e| {
            eprintln!("failed to run command: {}", e);
            e
        })?;
    Ok(status.code().unwrap_or_default())
}

fn job_start_inner(json_str: &str) -> Result<i32, JobError> {
    let cfg = serde_json::from_str::<JobConfig>(json_str)?;
    run_job(cfg)
}

#[no_mangle]
pub extern "C" fn job_start(config_json: *const c_char, exit_code: *mut c_int) -> bool {
    if config_json.is_null() || exit_code.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(config_json) };
    let json_str = match cstr.to_str() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("invalid utf-8 in config: {}", e);
            return false;
        }
    };
    match job_start_inner(json_str) {
        Ok(code) => {
            unsafe { *exit_code = code; }
            true
        }
        Err(e) => {
            eprintln!("job failed: {}", e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_true() {
        let cfg = JobConfig { cmd: "true".into(), args: vec![] };
        let code = run_job(cfg).expect("run true");
        assert_eq!(code, 0);
    }
}
