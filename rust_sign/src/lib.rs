use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_long};
use std::sync::Mutex;

#[derive(Clone, Debug)]
struct Sign {
    id: i32,
    name: String,
    lnum: i64,
}

static SIGNS: Lazy<Mutex<Vec<Sign>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[no_mangle]
pub extern "C" fn rs_sign_add(id: c_int, name: *const c_char, lnum: c_long) {
    if name.is_null() {
        return;
    }
    let c_str = unsafe { CStr::from_ptr(name) };
    if let Ok(name_str) = c_str.to_str() {
        let mut signs = SIGNS.lock().unwrap();
        signs.push(Sign { id, name: name_str.to_string(), lnum: lnum as i64 });
    }
}

#[no_mangle]
pub extern "C" fn rs_sign_delete(id: c_int) {
    let mut signs = SIGNS.lock().unwrap();
    signs.retain(|s| s.id != id);
}

#[no_mangle]
pub extern "C" fn rs_sign_draw() {
    let signs = SIGNS.lock().unwrap();
    for s in signs.iter() {
        println!("sign {} {} {}", s.id, s.name, s.lnum);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn add_delete_draw() {
        let name = CString::new("test").unwrap();
        rs_sign_add(1, name.as_ptr(), 10);
        assert_eq!(SIGNS.lock().unwrap().len(), 1);
        rs_sign_draw();
        rs_sign_delete(1);
        assert_eq!(SIGNS.lock().unwrap().len(), 0);
    }
}
