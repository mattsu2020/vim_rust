use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::sync::{Mutex, OnceLock};

#[cfg(any(feature = "x11", feature = "wayland", feature = "windows"))]
use copypasta::{ClipboardContext, ClipboardProvider};

#[cfg(not(any(feature = "x11", feature = "wayland", feature = "windows")))]
static MEMORY: OnceLock<Mutex<String>> = OnceLock::new();

#[cfg(not(any(feature = "x11", feature = "wayland", feature = "windows")))]
fn set_clipboard_text(s: &str) -> Result<(), ()> {
    *MEMORY.get_or_init(|| Mutex::new(String::new())).lock().unwrap() = s.to_string();
    Ok(())
}

#[cfg(not(any(feature = "x11", feature = "wayland", feature = "windows")))]
fn get_clipboard_text() -> Option<String> {
    Some(MEMORY.get_or_init(|| Mutex::new(String::new())).lock().unwrap().clone())
}

#[cfg(any(feature = "x11", feature = "wayland", feature = "windows"))]
fn set_clipboard_text(s: &str) -> Result<(), ()> {
    let mut ctx = ClipboardContext::new().map_err(|_| ())?;
    ctx.set_contents(s.to_string()).map_err(|_| ())
}

#[cfg(any(feature = "x11", feature = "wayland", feature = "windows"))]
fn get_clipboard_text() -> Option<String> {
    let mut ctx = ClipboardContext::new().ok()?;
    ctx.get_contents().ok()
}

#[no_mangle]
pub extern "C" fn rs_clipboard_set(data: *const c_char, len: usize) -> c_int {
    if data.is_null() {
        return -1;
    }
    let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
    let text = match std::str::from_utf8(slice) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    if set_clipboard_text(text).is_ok() { 0 } else { -1 }
}

#[no_mangle]
pub extern "C" fn rs_clipboard_get(len: *mut usize) -> *mut c_char {
    let text = get_clipboard_text().unwrap_or_default();
    unsafe {
        if !len.is_null() {
            *len = text.len();
        }
    }
    CString::new(text).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn rs_clipboard_free(ptr: *mut c_char, _len: usize) {
    if !ptr.is_null() {
        unsafe { let _ = CString::from_raw(ptr); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn roundtrip() {
        let s = CString::new("hello clipboard").unwrap();
        assert_eq!(rs_clipboard_set(s.as_ptr(), 15), 0);
        let mut len: usize = 0;
        let ptr = rs_clipboard_get(&mut len as *mut usize);
        assert_eq!(len, 15);
        let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
        assert_eq!(std::str::from_utf8(slice).unwrap(), "hello clipboard");
        rs_clipboard_free(ptr, len);
    }
}
