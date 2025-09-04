use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Format `text` using facilities from the `rust_text` crate and indent
/// according to `prev_line` using `rust_indent`.
///
/// The text is trimmed and converted to uppercase via `rs_format_text`.
/// The indentation is computed by `rs_cindent_level` based on `prev_line`.
/// If `prop_id` refers to a known text property, its name is prefixed inside
/// brackets.  The resulting string is returned as a newly allocated C string.
#[no_mangle]
pub extern "C" fn rs_format_with_indent(
    text: *const c_char,
    prev_line: *const c_char,
    prop_id: c_int,
) -> *mut c_char {
    if text.is_null() {
        return std::ptr::null_mut();
    }

    // Use rust_text to format the text (trim and uppercase).
    let formatted_ptr = rust_text::rs_format_text(text, true, true);
    if formatted_ptr.is_null() {
        return std::ptr::null_mut();
    }
    let formatted = unsafe { CStr::from_ptr(formatted_ptr) }
        .to_string_lossy()
        .to_string();
    rust_text::rs_free_cstring(formatted_ptr);

    // Determine indentation using rust_indent.
    let indent = if prev_line.is_null() {
        0
    } else {
        rust_indent::rs_cindent_level(prev_line) as usize
    };

    // Fetch the name of the text property, if any.
    let mut prefix = String::new();
    let prop_ptr = rust_text::rs_get_text_prop_name(prop_id);
    if !prop_ptr.is_null() {
        let name = unsafe { CStr::from_ptr(prop_ptr) }
            .to_string_lossy()
            .to_string();
        rust_text::rs_free_cstring(prop_ptr);
        prefix.push_str("[");
        prefix.push_str(&name);
        prefix.push_str("] ");
    }

    // Compose final string.
    let mut result = String::new();
    result.push_str(&" ".repeat(indent));
    result.push_str(&prefix);
    result.push_str(&formatted);

    CString::new(result).unwrap().into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn integrates_with_text_and_indent() {
        let prop_name = CString::new("note").unwrap();
        assert!(rust_text::rs_add_text_prop(1, prop_name.as_ptr()));

        let text = CString::new("  hello world  ").unwrap();
        let prev = CString::new("{").unwrap();
        let res_ptr = rs_format_with_indent(text.as_ptr(), prev.as_ptr(), 1);
        assert!(!res_ptr.is_null());
        let res = unsafe { CString::from_raw(res_ptr) };
        assert_eq!(res.to_str().unwrap(), "    [note] HELLO WORLD");
    }
}
