use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

use rust_optionstr::is_valid as option_string_is_valid;

mod bindings {
    include!("bindings.rs");
}
use bindings::rs_opt_t;
unsafe impl Sync for rs_opt_t {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptType {
    Bool,
    Number,
    String,
}

#[derive(Debug, Clone, Copy)]
pub struct OptionDef {
    pub id: OptionId,
    pub name: &'static str,
    pub short: &'static str,
    pub opt_type: OptType,
}

include!(concat!(env!("OUT_DIR"), "/option_defs.rs"));

#[derive(Debug, Clone, PartialEq, Eq)]
struct Opt {
    name: String,
    value: String,
}

fn parse_option(s: &str) -> Option<Opt> {
    let (name, value) = match s.split_once('=') {
        Some((n, v)) => (n, v),
        None => (s, "true"),
    };
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let value = value.trim();
    Some(Opt {
        name: name.to_string(),
        value: value.to_string(),
    })
}

static OPTIONS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn options() -> &'static Mutex<HashMap<String, String>> {
    OPTIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[no_mangle]
pub extern "C" fn rs_options_init() {
    let mut opts = options().lock().unwrap();
    if opts.is_empty() {
        for opt in OPTION_TABLE.iter() {
            opts.insert(opt.name.to_string(), String::new());
        }
    }
}

fn apply_option(name: &str, value: &str) -> bool {
    let name = name.trim();
    if name.is_empty() {
        return false;
    }
    let value = value.trim();
    if !option_string_is_valid(name, value) {
        return false;
    }
    options()
        .lock()
        .unwrap()
        .insert(name.to_string(), value.to_string());
    true
}

#[no_mangle]
pub extern "C" fn rs_apply_option(name: *const c_char, value: *const c_char) -> bool {
    if name.is_null() || value.is_null() {
        return false;
    }
    let name = unsafe { CStr::from_ptr(name) };
    let value = unsafe { CStr::from_ptr(value) };
    let Ok(name) = name.to_str() else {
        return false;
    };
    let Ok(value) = value.to_str() else {
        return false;
    };
    apply_option(name, value)
}

#[no_mangle]
pub extern "C" fn rs_set_option(name: *const c_char, value: *const c_char) -> bool {
    rs_apply_option(name, value)
}

#[no_mangle]
pub extern "C" fn rs_get_option(name: *const c_char) -> *mut c_char {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    let name = unsafe { CStr::from_ptr(name) };
    let key = match name.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return std::ptr::null_mut(),
    };
    if let Some(val) = options().lock().unwrap().get(&key) {
        if let Ok(cs) = CString::new(val.as_str()) {
            return cs.into_raw();
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn rs_free_cstring(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_get_option_defs(len: *mut usize) -> *const rs_opt_t {
    if !len.is_null() {
        unsafe {
            *len = OPTION_DEFS.len();
        }
    }
    OPTION_DEFS.as_ptr()
}

#[no_mangle]
pub extern "C" fn rs_verify_option(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(name) };
    let Ok(name) = cstr.to_str() else {
        return false;
    };
    OPTION_TABLE
        .iter()
        .any(|opt| opt.name == name || (!opt.short.is_empty() && opt.short == name))
}

#[no_mangle]
pub extern "C" fn rs_save_options(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let cpath = unsafe { CStr::from_ptr(path) };
    let Ok(path) = cpath.to_str() else {
        return false;
    };
    let opts = options().lock().unwrap();
    let mut file = match std::fs::File::create(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    for (k, v) in opts.iter() {
        if std::io::Write::write_all(&mut file, format!("{}={}\n", k, v).as_bytes()).is_err() {
            return false;
        }
    }
    true
}

#[no_mangle]
pub extern "C" fn rs_load_options(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let cpath = unsafe { CStr::from_ptr(path) };
    let Ok(path) = cpath.to_str() else {
        return false;
    };
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let mut opts = options().lock().unwrap();
    opts.clear();
    for line in content.lines() {
        if let Some((k, v)) = line.split_once('=') {
            opts.insert(k.to_string(), v.to_string());
        }
    }
    true
}

#[no_mangle]
pub extern "C" fn rs_parse_option(assignment: *const c_char) -> bool {
    if assignment.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(assignment) };
    let Ok(s) = cstr.to_str() else { return false };
    if let Some(opt) = parse_option(s) {
        let name = match CString::new(opt.name) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let value = match CString::new(opt.value) {
            Ok(c) => c,
            Err(_) => return false,
        };
        rs_apply_option(name.as_ptr(), value.as_ptr())
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options() {
        rs_options_init();
        let name = CString::new("shell").unwrap();
        let val_ptr = rs_get_option(name.as_ptr());
        assert!(!val_ptr.is_null());
        unsafe {
            drop(CString::from_raw(val_ptr));
        }
    }

    #[test]
    fn set_and_get() {
        rs_options_init();
        let name = CString::new("testopt").unwrap();
        let value = CString::new("123").unwrap();
        assert!(rs_set_option(name.as_ptr(), value.as_ptr()));
        let res_ptr = rs_get_option(name.as_ptr());
        assert!(!res_ptr.is_null());
        let res = unsafe { CString::from_raw(res_ptr) };
        assert_eq!(res.to_str().unwrap(), "123");
    }

    #[test]
    fn verify_option() {
        rs_options_init();
        let good = CString::new("shell").unwrap();
        assert!(rs_verify_option(good.as_ptr()));
        let bad = CString::new("no_such_opt").unwrap();
        assert!(!rs_verify_option(bad.as_ptr()));
    }

    #[test]
    fn save_and_load() {
        rs_options_init();
        let name = CString::new("saveopt").unwrap();
        let value = CString::new("foo").unwrap();
        assert!(rs_set_option(name.as_ptr(), value.as_ptr()));
        let file = tempfile::NamedTempFile::new().unwrap();
        let path = CString::new(file.path().to_str().unwrap()).unwrap();
        assert!(rs_save_options(path.as_ptr()));
        options().lock().unwrap().clear();
        assert!(rs_load_options(path.as_ptr()));
        let res_ptr = rs_get_option(name.as_ptr());
        assert!(!res_ptr.is_null());
        let res = unsafe { CString::from_raw(res_ptr) };
        assert_eq!(res.to_str().unwrap(), "foo");
    }

    #[test]
    fn parse_option_assignment() {
        rs_options_init();
        let assign = CString::new("background=dark").unwrap();
        assert!(rs_parse_option(assign.as_ptr()));
        let name = CString::new("background").unwrap();
        let val_ptr = rs_get_option(name.as_ptr());
        assert!(!val_ptr.is_null());
        let val = unsafe { CString::from_raw(val_ptr) };
        assert_eq!(val.to_str().unwrap(), "dark");
    }

    #[test]
    fn apply_option_direct() {
        rs_options_init();
        let name = CString::new("direct").unwrap();
        let value = CString::new("42").unwrap();
        assert!(rs_apply_option(name.as_ptr(), value.as_ptr()));
        let res_ptr = rs_get_option(name.as_ptr());
        assert!(!res_ptr.is_null());
        let res = unsafe { CString::from_raw(res_ptr) };
        assert_eq!(res.to_str().unwrap(), "42");
    }

    #[test]
    fn parse_option_invalid() {
        let assign = CString::new("=bad").unwrap();
        assert!(!rs_parse_option(assign.as_ptr()));
    }

    #[test]
    fn validate_string_option() {
        rs_options_init();
        let name = CString::new("background").unwrap();
        let good = CString::new("dark").unwrap();
        assert!(rs_set_option(name.as_ptr(), good.as_ptr()));
        let bad = CString::new("blue").unwrap();
        assert!(!rs_set_option(name.as_ptr(), bad.as_ptr()));
    }
}
