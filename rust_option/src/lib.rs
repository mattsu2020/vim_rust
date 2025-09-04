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

pub fn options_init() {
    let mut opts = options().lock().unwrap();
    if opts.is_empty() {
        for opt in OPTION_TABLE.iter() {
            opts.insert(opt.name.to_string(), String::new());
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_options_init() {
    options_init();
}

pub fn set_option(name: &str, value: &str) -> bool {
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

pub fn get_option(name: &str) -> Option<String> {
    options().lock().unwrap().get(name).cloned()
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
    set_option(name, value)
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
    let Ok(key) = name.to_str() else {
        return std::ptr::null_mut();
    };
    if let Some(val) = get_option(key) {
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

pub fn verify_option(name: &str) -> bool {
    OPTION_TABLE
        .iter()
        .any(|opt| opt.name == name || (!opt.short.is_empty() && opt.short == name))
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
    verify_option(name)
}

pub fn save_options(path: &str) -> bool {
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
pub extern "C" fn rs_save_options(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let cpath = unsafe { CStr::from_ptr(path) };
    let Ok(path) = cpath.to_str() else {
        return false;
    };
    save_options(path)
}

pub fn load_options(path: &str) -> bool {
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
pub extern "C" fn rs_load_options(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let cpath = unsafe { CStr::from_ptr(path) };
    let Ok(path) = cpath.to_str() else {
        return false;
    };
    load_options(path)
}

pub fn parse_option_assignment(assignment: &str) -> bool {
    if let Some(opt) = parse_option(assignment) {
        set_option(&opt.name, &opt.value)
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn rs_parse_option(assignment: *const c_char) -> bool {
    if assignment.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(assignment) };
    let Ok(s) = cstr.to_str() else { return false };
    parse_option_assignment(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options() {
        options_init();
        assert!(get_option("shell").is_some());
    }

    #[test]
    fn set_and_get() {
        options_init();
        assert!(set_option("testopt", "123"));
        assert_eq!(get_option("testopt").unwrap(), "123");
    }

    #[test]
    fn verify_option_works() {
        options_init();
        assert!(verify_option("shell"));
        assert!(!verify_option("no_such_opt"));
    }

    #[test]
    fn save_and_load() {
        options_init();
        assert!(set_option("saveopt", "foo"));
        let file = tempfile::NamedTempFile::new().unwrap();
        assert!(save_options(file.path().to_str().unwrap()));
        options().lock().unwrap().clear();
        assert!(load_options(file.path().to_str().unwrap()));
        assert_eq!(get_option("saveopt").unwrap(), "foo");
    }

    #[test]
    fn parse_option_assignment_sets_value() {
        options_init();
        assert!(parse_option_assignment("background=dark"));
        assert_eq!(get_option("background").unwrap(), "dark");
    }

    #[test]
    fn apply_option_direct() {
        options_init();
        assert!(set_option("direct", "42"));
        assert_eq!(get_option("direct").unwrap(), "42");
    }

    #[test]
    fn parse_option_invalid() {
        options_init();
        assert!(!parse_option_assignment("=bad"));
    }

    #[test]
    fn validate_string_option() {
        options_init();
        assert!(set_option("background", "dark"));
        assert!(!set_option("background", "blue"));
    }
}
