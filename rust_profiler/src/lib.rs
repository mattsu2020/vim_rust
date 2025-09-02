use std::ffi::CString;
use std::os::raw::{c_char, c_long};
use std::time::Instant;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static START: Lazy<Mutex<Option<Instant>>> = Lazy::new(|| Mutex::new(None));

#[no_mangle]
pub extern "C" fn rs_profiler_start() {
    let mut s = START.lock().unwrap();
    *s = Some(Instant::now());
}

#[no_mangle]
pub extern "C" fn rs_profiler_stop() -> c_long {
    let mut s = START.lock().unwrap();
    if let Some(start) = s.take() {
        let dur = start.elapsed();
        dur.as_micros() as c_long
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn rs_profiler_format(micros: c_long) -> *mut c_char {
    let secs = micros / 1_000_000;
    let rem = micros % 1_000_000;
    let s = format!("{}.{:06} sec", secs, rem);
    CString::new(s).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn rs_profiler_string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::time::Duration;

    #[test]
    fn measure_time() {
        rs_profiler_start();
        std::thread::sleep(Duration::from_millis(1));
        let us = rs_profiler_stop();
        assert!(us > 0);
    }

    #[test]
    fn format_output() {
        let ptr = rs_profiler_format(123456);
        let c = unsafe { CStr::from_ptr(ptr) };
        assert_eq!(c.to_str().unwrap(), "0.123456 sec");
        rs_profiler_string_free(ptr);
    }
}
