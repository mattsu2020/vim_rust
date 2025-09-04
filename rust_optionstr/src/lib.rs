use std::ffi::CStr;
use std::os::raw::c_char;

#[derive(Debug)]
struct StringOptionDef {
    name: &'static str,
    values: &'static [&'static str],
}

impl StringOptionDef {
    fn is_valid(&self, value: &str) -> bool {
        self.values.iter().any(|&v| {
            if let Some(prefix) = v.strip_suffix(':') {
                value.starts_with(prefix)
            } else {
                v == value
            }
        })
    }
}

static STRING_OPTIONS: &[StringOptionDef] = &[
    StringOptionDef {
        name: "background",
        values: &["light", "dark"],
    },
    StringOptionDef {
        name: "fileformat",
        values: &["unix", "dos", "mac"],
    },
    StringOptionDef {
        name: "clipboard",
        values: &[
            "unnamed",
            "unnamedplus",
            "autoselect",
            "autoselectplus",
            "autoselectml",
            "html",
            "exclude:",
        ],
    },
];

/// Returns true if the option is either unknown or the value is valid for the
/// known option.
pub fn is_valid(name: &str, value: &str) -> bool {
    if let Some(def) = STRING_OPTIONS.iter().find(|d| d.name == name) {
        def.is_valid(value)
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn rs_option_string_is_valid(
    name: *const c_char,
    value: *const c_char,
) -> bool {
    if name.is_null() || value.is_null() {
        return false;
    }
    let name = unsafe { CStr::from_ptr(name) };
    let value = unsafe { CStr::from_ptr(value) };
    let Ok(name) = name.to_str() else { return false };
    let Ok(value) = value.to_str() else { return false };
    is_valid(name, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_known_option() {
        assert!(is_valid("background", "dark"));
        assert!(!is_valid("background", "blue"));
    }

    #[test]
    fn unknown_option_always_valid() {
        assert!(is_valid("doesnotexist", "whatever"));
    }
}
