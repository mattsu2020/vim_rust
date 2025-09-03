use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_long, c_void};
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[repr(C)]
pub struct typval_T { _private: [u8; 0] }

#[derive(Default)]
struct QfEntry {
    text: String,
    lnum: i64,
    col: i32,
}

static LIST: Lazy<Mutex<Vec<QfEntry>>> = Lazy::new(|| Mutex::new(Vec::new()));

fn parse_error_line(line: &str) -> Option<QfEntry> {
    let mut parts = line.splitn(4, ':');
    let _file = parts.next()?;
    let lnum = parts.next()?.trim().parse().ok()?;
    let col = parts.next()?.trim().parse().ok()?;
    let text = parts.next()?.trim().to_string();
    Some(QfEntry { text, lnum, col })
}

#[no_mangle]
pub extern "C" fn rs_qf_add_entry(
    _qfl: *mut c_void,
    _dir: *const c_char,
    _fname: *const c_char,
    _module: *const c_char,
    _bufnum: c_int,
    mesg: *const c_char,
    lnum: c_long,
    _end_lnum: c_long,
    col: c_int,
    _end_col: c_int,
    _vis_col: c_int,
    _pattern: *const c_char,
    _nr: c_int,
    _typ: c_int,
    _user_data: *mut typval_T,
    _valid: c_int,
) -> c_int {
    if mesg.is_null() {
        return 0; // QF_FAIL
    }
    let c_str = unsafe { CStr::from_ptr(mesg) };
    let text = match c_str.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return 0,
    };
    let mut list = LIST.lock().unwrap();
    list.push(QfEntry { text, lnum: lnum as i64, col });
    1 // QF_OK
}

#[no_mangle]
pub extern "C" fn rs_qf_list(_eap: *mut c_void) {
    let list = LIST.lock().unwrap();
    for (i, item) in list.iter().enumerate() {
        println!("{}: {}:{} {}", i + 1, item.lnum, item.col, item.text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn add_and_list() {
        let msg = CString::new("test").unwrap();
        assert_eq!(rs_qf_add_entry(std::ptr::null_mut(), std::ptr::null(), std::ptr::null(), std::ptr::null(), 0, msg.as_ptr(), 1, 0, 0, 0, 0, std::ptr::null(), 0, 0, std::ptr::null_mut(), 1), 1);
        rs_qf_list(std::ptr::null_mut());
    }

    #[test]
    fn parse_error() {
        let line = "main.rs:10:5: undefined variable";
        let e = parse_error_line(line).expect("parsed");
        assert_eq!(e.lnum, 10);
        assert_eq!(e.col, 5);
        assert_eq!(e.text, "undefined variable");
    }

    #[test]
    fn parse_error_invalid() {
        assert!(parse_error_line("nonsense").is_none());
    }
}
