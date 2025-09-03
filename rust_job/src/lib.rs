use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use tokio::process::Command;
use tokio::runtime::Runtime;

#[derive(Debug, Deserialize)]
pub struct JobConfig {
    pub cmd: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug)]
pub enum JobError {
    Json(serde_json::Error),
    Io(std::io::Error),
}

impl From<serde_json::Error> for JobError {
    fn from(e: serde_json::Error) -> Self { JobError::Json(e) }
}

impl From<std::io::Error> for JobError {
    fn from(e: std::io::Error) -> Self { JobError::Io(e) }
}

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn run_job(config: JobConfig) -> Result<i32, JobError> {
    let rt = RUNTIME.get_or_init(|| Runtime::new().unwrap());
    let status = rt.block_on(async {
        Command::new(&config.cmd).args(&config.args).status().await
    })?;
    Ok(status.code().unwrap_or_default())
}

#[no_mangle]
pub extern "C" fn job_start(config_json: *const c_char, exit_code: *mut c_int) -> bool {
    if config_json.is_null() || exit_code.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(config_json) };
    let json_str = match cstr.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    match serde_json::from_str::<JobConfig>(json_str) {
        Ok(cfg) => match run_job(cfg) {
            Ok(code) => {
                unsafe { *exit_code = code; }
                true
            }
            Err(_) => false,
        },
        Err(_) => false,
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
