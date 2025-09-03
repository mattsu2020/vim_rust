mod ast;
mod executor;
mod types;

use std::ffi::CStr;
use std::os::raw::c_char;

/// Parse and execute a Vim9 class declaration.
/// Returns a small integer based on the class name length, or -1 on error.
#[no_mangle]
pub extern "C" fn rs_vim9class_eval(src: *const c_char) -> i32 {
    if src.is_null() {
        return -1;
    }
    let cstr = unsafe { CStr::from_ptr(src) };
    let text = match cstr.to_str() {
        Ok(t) => t,
        Err(_) => return -1,
    };
    let class = ast::parse(text);
    let _ty = types::type_of(&class);
    executor::execute(&class)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn eval_returns_name_length() {
        let src = CString::new("class Foo").unwrap();
        let res = rs_vim9class_eval(src.as_ptr());
        assert_eq!(res, 3);
    }
}
