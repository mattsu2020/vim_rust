use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

#[derive(Debug, PartialEq)]
pub enum FFIError {
    Null,
    Utf8,
    Exec,
    Init,
}

pub type FFIResult<T> = Result<T, FFIError>;

pub fn cstr_to_str<'a>(ptr: *const c_char) -> FFIResult<&'a str> {
    if ptr.is_null() {
        return Err(FFIError::Null);
    }
    unsafe { CStr::from_ptr(ptr).to_str().map_err(|_| FFIError::Utf8) }
}

pub fn result_to_int(res: FFIResult<()>) -> c_int {
    res.map_or(0, |_| 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn convert_valid_str() {
        let c = CString::new("ok").unwrap();
        assert_eq!(cstr_to_str(c.as_ptr()).unwrap(), "ok");
    }

    #[test]
    fn null_pointer_error() {
        let err = cstr_to_str(std::ptr::null()).unwrap_err();
        assert_eq!(err, FFIError::Null);
    }
}
